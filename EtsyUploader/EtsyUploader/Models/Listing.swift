//
//  Listing.swift
//  EtsyUploader
//
//  Created by Zach Baron on 9/21/26.
//

import Foundation

enum ProductType: String, Codable, CaseIterable, Identifiable {
    case pipe
    case tamper
    case ashtray

    var id: String {
        rawValue
    }

    var displayName: String {
        switch self {
        case .pipe:
            return "Pipe"

        case .tamper:
            return "Tamper"

        case .ashtray:
            return "Ashtray"
        }
    }
}

struct PipeDimensions: Codable {
    var overallLength: Double
    var bowlHeight: Double
    var chamberDiameter: Double
    var chamberDepth: Double

    enum CodingKeys: String, CodingKey {
        case overallLength = "overall_length"
        case bowlHeight = "bowl_height"
        case chamberDiameter = "chamber_diameter"
        case chamberDepth = "chamber_depth"
    }
}

struct AccessoryDimensions: Codable {
    var length: Double
    var width: Double
    var height: Double
}

enum ProductDimensions: Codable {
    case pipe(PipeDimensions)
    case accessory(AccessoryDimensions)

    enum CodingKeys: String, CodingKey {
        case type
        case dimensions
    }

    enum DimensionType: String, Codable {
        case pipe = "Pipe"
        case accessory = "Accessory"
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(
            keyedBy: CodingKeys.self
        )

        switch self {
        case .pipe(let dimensions):
            try container.encode(
                DimensionType.pipe,
                forKey: .type
            )

            try container.encode(
                dimensions,
                forKey: .dimensions
            )

        case .accessory(let dimensions):
            try container.encode(
                DimensionType.accessory,
                forKey: .type
            )

            try container.encode(
                dimensions,
                forKey: .dimensions
            )
        }
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(
            keyedBy: CodingKeys.self
        )

        let type = try container.decode(
            DimensionType.self,
            forKey: .type
        )

        switch type {
        case .pipe:
            let dimensions = try container.decode(
                PipeDimensions.self,
                forKey: .dimensions
            )

            self = .pipe(dimensions)

        case .accessory:
            let dimensions = try container.decode(
                AccessoryDimensions.self,
                forKey: .dimensions
            )

            self = .accessory(dimensions)
        }
    }
}

struct PackageDimensions: Codable {
    var length: Double
    var width: Double
    var height: Double
    var weightOz: Double

    enum CodingKeys: String, CodingKey {
        case length
        case width
        case height
        case weightOz = "weight_oz"
    }
}

struct CreateListingRequest: Codable {
    var productType: ProductType
    var title: String
    var price: Double
    var description: String
    var materials: [String]
    var productDimensions: ProductDimensions
    var packageDimensions: PackageDimensions

    enum CodingKeys: String, CodingKey {
        case productType = "product_type"
        case title
        case price
        case description
        case materials
        case productDimensions = "product_dimensions"
        case packageDimensions = "package_dimensions"
    }
}
