use std::path::PathBuf;

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    models::listing::CreateListingRequest,
    services::etsy::{
        images::upload_listing_image,
        listings::create_draft_listing,
        shops::get_my_shop,
    },
};

struct UploadedImage {
    path: PathBuf,
}

pub async fn create_listing(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut listing: Option<CreateListingRequest> = None;
    let mut images: Vec<UploadedImage> = Vec::new();

    while let Some(field) = match multipart.next_field().await {
        Ok(field) => field,

        Err(error) => {
            cleanup_images(&images).await;

            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!(
                        "Failed to parse multipart request: {error}"
                    ),
                })),
            )
                .into_response();
        }
    } {
        let field_name = field
            .name()
            .unwrap_or("")
            .to_string();

        match field_name.as_str() {
            "listing" => {
                let json = match field.text().await {
                    Ok(json) => json,

                    Err(error) => {
                        cleanup_images(&images).await;

                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({
                                "success": false,
                                "error": format!(
                                    "Failed to read listing data: {error}"
                                ),
                            })),
                        )
                            .into_response();
                    }
                };

                let parsed =
                    match serde_json::from_str::<CreateListingRequest>(
                        &json,
                    ) {
                        Ok(listing) => listing,

                        Err(error) => {
                            cleanup_images(&images).await;

                            return (
                                StatusCode::BAD_REQUEST,
                                Json(serde_json::json!({
                                    "success": false,
                                    "error": format!(
                                        "Invalid listing JSON: {error}"
                                    ),
                                })),
                            )
                                .into_response();
                        }
                    };

                listing = Some(parsed);
            }

            "images" => {
                let original_file_name = field
                    .file_name()
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        "listing-image.jpg".to_string()
                    });

                let bytes = match field.bytes().await {
                    Ok(bytes) => bytes,

                    Err(error) => {
                        cleanup_images(&images).await;

                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({
                                "success": false,
                                "error": format!(
                                    "Failed to read image: {error}"
                                ),
                            })),
                        )
                            .into_response();
                    }
                };

                let unique_file_name = format!(
                    "{}-{}",
                    images.len(),
                    original_file_name
                );

                let path = PathBuf::from("/tmp")
                    .join(unique_file_name);

                if let Err(error) =
                    tokio::fs::write(&path, bytes).await
                {
                    cleanup_images(&images).await;

                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": format!(
                                "Failed to temporarily store image: {error}"
                            ),
                        })),
                    )
                        .into_response();
                }

                images.push(UploadedImage {
                    path,
                });
            }

            _ => {}
        }
    }

    let listing = match listing {
        Some(listing) => listing,

        None => {
            cleanup_images(&images).await;

            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Missing listing data",
                })),
            )
                .into_response();
        }
    };

    if images.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "At least one image is required",
            })),
        )
            .into_response();
    }

    /*
        Resolve the shop BEFORE creating the draft.

        That way, if shop discovery fails, we haven't created
        an orphan Etsy draft.
    */

    let shop = match get_my_shop().await {
        Ok(shop) => shop,

        Err(error) => {
            let error_message = error.to_string();

            cleanup_images(&images).await;

            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!(
                        "Failed to retrieve Etsy shop: {error_message}"
                    ),
                })),
            )
                .into_response();
        }
    };

    /*
        Create the Etsy draft.

        From this point onward, a listing actually exists on Etsy.
    */

    let draft = match create_draft_listing(&listing).await {
        Ok(draft) => draft,

        Err(error) => {
            let error_message = error.to_string();

            cleanup_images(&images).await;

            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "success": false,
                    "error": error_message,
                })),
            )
                .into_response();
        }
    };

    let mut uploaded_image_ids: Vec<u64> = Vec::new();

    /*
        Upload sequentially.

        Do NOT make these concurrent.

        The order of this Vec is the order supplied by the client,
        and we're intentionally preserving that order for Etsy.
    */

    for (index, image) in images.iter().enumerate() {
        match upload_listing_image(
            shop.shop_id,
            draft.listing_id,
            &image.path,
        )
        .await
        {
            Ok(uploaded_image) => {
                uploaded_image_ids.push(
                    uploaded_image.listing_image_id,
                );

                println!(
                    "Uploaded image {} of {} to Etsy listing {}",
                    index + 1,
                    images.len(),
                    draft.listing_id
                );
            }

            Err(error) => {
                let error_message = error.to_string();

                cleanup_images(&images).await;

                /*
                    IMPORTANT:

                    We return the listing_id even though this request
                    failed overall.

                    The Etsy draft already exists at this point.

                    This prevents the client from assuming nothing
                    happened and blindly creating another draft.
                */

                return (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({
                        "success": false,
                        "partial_success": true,
                        "listing_id": draft.listing_id,
                        "uploaded_images": uploaded_image_ids.len(),
                        "failed_image_index": index,
                        "error": format!(
                            "Draft was created, but image {} failed to upload: {}",
                            index + 1,
                            error_message
                        ),
                    })),
                )
                    .into_response();
            }
        }
    }

    cleanup_images(&images).await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "listing_id": draft.listing_id,
            "title": draft.title,
            "state": draft.state,
            "uploaded_images": uploaded_image_ids.len(),
            "image_ids": uploaded_image_ids,
        })),
    )
        .into_response()
}

async fn cleanup_images(
    images: &[UploadedImage],
) {
    for image in images {
        if let Err(error) =
            tokio::fs::remove_file(&image.path).await
        {
            eprintln!(
                "Failed to remove temporary image {:?}: {}",
                image.path,
                error
            );
        }
    }
}