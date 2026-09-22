//
//  ListingResponse.swift
//  EtsyUploader
//
//  Created by Zach Baron on 9/21/26.
//

import Foundation

struct CreateListingResponse: Decodable {
    let success: Bool
    let listingId: UInt64?
    let title: String?
    let state: String?
    let uploadedImages: Int?
    let imageIds: [UInt64]?

    let partialSuccess: Bool?
    let failedImageIndex: Int?
    let error: String?

    enum CodingKeys: String, CodingKey {
        case success
        case listingId = "listing_id"
        case title
        case state
        case uploadedImages = "uploaded_images"
        case imageIds = "image_ids"

        case partialSuccess = "partial_success"
        case failedImageIndex = "failed_image_index"
        case error
    }
}
