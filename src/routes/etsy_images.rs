use std::path::PathBuf;

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::services::etsy::{
    images::upload_listing_image,
    shops::get_my_shop,
};

pub async fn test_upload_image(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut listing_id: Option<u64> = None;
    let mut image_path: Option<PathBuf> = None;

    while let Some(field) = match multipart.next_field().await {
        Ok(field) => field,

        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": error.to_string(),
                })),
            )
                .into_response();
        }
    } {
        let name = field
            .name()
            .unwrap_or("")
            .to_string();

        match name.as_str() {
            "listing_id" => {
                let value = match field.text().await {
                    Ok(value) => value,

                    Err(error) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({
                                "success": false,
                                "error": error.to_string(),
                            })),
                        )
                            .into_response();
                    }
                };

                listing_id = match value.parse::<u64>() {
                    Ok(id) => Some(id),

                    Err(_) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({
                                "success": false,
                                "error": "Invalid listing_id",
                            })),
                        )
                            .into_response();
                    }
                };
            }

            "image" => {
                let file_name = field
                    .file_name()
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        "listing-image.jpg".to_string()
                    });

                let bytes = match field.bytes().await {
                    Ok(bytes) => bytes,

                    Err(error) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({
                                "success": false,
                                "error": error.to_string(),
                            })),
                        )
                            .into_response();
                    }
                };

                let path = PathBuf::from(format!(
                    "/tmp/{file_name}"
                ));

                if let Err(error) =
                    tokio::fs::write(&path, &bytes).await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "success": false,
                            "error": error.to_string(),
                        })),
                    )
                        .into_response();
                }

                image_path = Some(path);
            }

            _ => {}
        }
    }

    let listing_id = match listing_id {
        Some(id) => id,

        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Missing listing_id",
                })),
            )
                .into_response();
        }
    };

    let image_path = match image_path {
        Some(path) => path,

        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Missing image",
                })),
            )
                .into_response();
        }
    };

    let shop = match get_my_shop().await {
        Ok(shop) => shop,

        Err(error) => {
            let error_message = error.to_string();

            let _ =
                tokio::fs::remove_file(&image_path).await;

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

    let result = upload_listing_image(
        shop.shop_id,
        listing_id,
        &image_path,
    )
    .await
    .map_err(|error| error.to_string());

    // We no longer need the temporary local copy,
    // regardless of whether Etsy accepted the image.
    let _ =
        tokio::fs::remove_file(&image_path).await;

    match result {
        Ok(image) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "listing_id": image.listing_id,
                "listing_image_id": image.listing_image_id,
                "rank": image.rank,
            })),
        )
            .into_response(),

        Err(error) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "success": false,
                "error": error,
            })),
        )
            .into_response(),
    }
}