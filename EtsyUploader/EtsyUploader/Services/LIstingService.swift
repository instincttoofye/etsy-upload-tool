//
//  LIstingService.swift
//  EtsyUploader
//
//  Created by Zach Baron on 9/21/26.
//

import Foundation

enum ListingServiceError: LocalizedError {
    case invalidResponse
    case serverError(String)

    var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "The server returned an invalid response."

        case .serverError(let message):
            return message
        }
    }
}

final class ListingService {
    private let baseURL = URL(
        string: "https://etsy-upload-tool-backend.fly.dev"
    )!

    func createListing(
        listing: CreateListingRequest,
        imageURLs: [URL]
    ) async throws -> CreateListingResponse {
        let boundary = UUID().uuidString

        let url = baseURL.appendingPathComponent("listings")

        var request = URLRequest(url: url)

        request.httpMethod = "POST"

        request.setValue(
            "multipart/form-data; boundary=\(boundary)",
            forHTTPHeaderField: "Content-Type"
        )

        let body = try buildMultipartBody(
            listing: listing,
            imageURLs: imageURLs,
            boundary: boundary
        )

        request.httpBody = body

        let (data, response) = try await URLSession.shared.data(
            for: request
        )

        guard let httpResponse = response as? HTTPURLResponse else {
            throw ListingServiceError.invalidResponse
        }

        let decoder = JSONDecoder()

        let listingResponse: CreateListingResponse

        do {
            listingResponse = try decoder.decode(
                CreateListingResponse.self,
                from: data
            )
        } catch {
            let body =
                String(data: data, encoding: .utf8)
                ?? "<unreadable response>"

            print("Backend response:")
            print(body)

            throw ListingServiceError.invalidResponse
        }

        guard (200...299).contains(httpResponse.statusCode) else {
            throw ListingServiceError.serverError(
                listingResponse.error
                    ?? "Backend returned HTTP \(httpResponse.statusCode)"
            )
        }

        return listingResponse
    }

    private func buildMultipartBody(
        listing: CreateListingRequest,
        imageURLs: [URL],
        boundary: String
    ) throws -> Data {
        var body = Data()

        let encoder = JSONEncoder()
        let listingData = try encoder.encode(listing)

        // Listing JSON field
        body.appendMultipartString("--\(boundary)\r\n")
        body.appendMultipartString(
            "Content-Disposition: form-data; name=\"listing\"\r\n"
        )
        body.appendMultipartString(
            "Content-Type: application/json\r\n"
        )
        body.appendMultipartString("\r\n")
        body.append(listingData)
        body.appendMultipartString("\r\n")

        // Image fields
        for imageURL in imageURLs {
            let didAccess =
                imageURL.startAccessingSecurityScopedResource()

            defer {
                if didAccess {
                    imageURL.stopAccessingSecurityScopedResource()
                }
            }

            let imageData = try Data(contentsOf: imageURL)
            let fileName = imageURL.lastPathComponent
            let mimeType = mimeType(for: imageURL)

            body.appendMultipartString("--\(boundary)\r\n")
            body.appendMultipartString(
                "Content-Disposition: form-data; name=\"images\"; filename=\"\(fileName)\"\r\n"
            )
            body.appendMultipartString(
                "Content-Type: \(mimeType)\r\n"
            )
            body.appendMultipartString("\r\n")
            body.append(imageData)
            body.appendMultipartString("\r\n")
        }

        // Closing boundary
        body.appendMultipartString("--\(boundary)--\r\n")

        return body
    }
    
    private func mimeType(
        for url: URL
    ) -> String {
        switch url.pathExtension.lowercased() {
        case "jpg", "jpeg":
            return "image/jpeg"

        case "png":
            return "image/png"

        case "heic":
            return "image/heic"

        default:
            return "application/octet-stream"
        }
    }
}

private extension Data {
    mutating func appendMultipartString(
        _ string: String
    ) {
        guard let data = string.data(
            using: .utf8
        ) else {
            return
        }

        append(data)
    }
}
