//
//  CreateListingView.swift
//  EtsyUploader
//
//  Created by Zach Baron on 9/21/26.
//

import SwiftUI
import UniformTypeIdentifiers

struct CreateListingView: View {
    @State private var title = ""
    @State private var price = ""
    @State private var description = ""
    @State private var materials = ""
    @State private var productType: ProductType = .pipe

    @State private var overallLength = ""
    @State private var bowlHeight = ""
    @State private var chamberDiameter = ""
    @State private var chamberDepth = ""
    
    @State private var accessoryLength = ""
    @State private var accessoryWidth = ""
    @State private var accessoryHeight = ""

    @State private var packageLength = ""
    @State private var packageWidth = ""
    @State private var packageHeight = ""
    @State private var packageWeight = ""

    @State private var imageURLs: [URL] = []

    @State private var showingImageImporter = false
    @State private var isUploading = false

    @State private var statusMessage = ""

    private let listingService = ListingService()

    var body: some View {
        ScrollView {
            VStack(
                alignment: .leading,
                spacing: 20
            ) {
                Text("InstinctivePipings")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("Create Etsy Draft")
                    .font(.title2)

                Divider()

                listingSection

                Divider()

                productDimensionsSection

                Divider()

                packageSection

                Divider()

                photosSection

                Divider()

                Button {
                    createListing()
                } label: {
                    if isUploading {
                        ProgressView()
                            .controlSize(.small)
                    } else {
                        Text("Create Etsy Draft")
                    }
                }
                .disabled(
                    isUploading ||
                    imageURLs.isEmpty
                )

                if !statusMessage.isEmpty {
                    Text(statusMessage)
                        .textSelection(.enabled)
                }
            }
            .padding(30)
            .frame(maxWidth: 700)
        }
        .fileImporter(
            isPresented: $showingImageImporter,
            allowedContentTypes: [.image],
            allowsMultipleSelection: true
        ) { result in
            switch result {
            case .success(let urls):
                imageURLs.append(
                    contentsOf: urls
                )

            case .failure(let error):
                statusMessage =
                    "Photo selection failed: \(error.localizedDescription)"
            }
        }
    }

    private var listingSection: some View {
        VStack(
            alignment: .leading,
            spacing: 12
        ) {
            Text("Listing")
                .font(.headline)
            
            Picker(
                "Product Type",
                selection: $productType
            ) {
                ForEach(ProductType.allCases) { type in
                    Text(type.displayName)
                        .tag(type)
                }
            }
            .pickerStyle(.segmented)

            TextField(
                "Title",
                text: $title
            )

            TextField(
                "Price",
                text: $price
            )

            TextField(
                "Materials — comma separated",
                text: $materials
            )

            TextEditor(
                text: $description
            )
            .frame(minHeight: 150)
            .overlay {
                RoundedRectangle(cornerRadius: 5)
                    .stroke(.secondary.opacity(0.3))
            }
        }
    }

    @ViewBuilder
    private var productDimensionsSection: some View {
        switch productType {
        case .pipe:
            pipeDimensionsSection

        case .tamper, .ashtray:
            accessoryDimensionsSection
        }
    }

    private var pipeDimensionsSection: some View {
        VStack(
            alignment: .leading,
            spacing: 12
        ) {
            Text("Pipe Dimensions")
                .font(.headline)

            TextField(
                "Overall Length",
                text: $overallLength
            )

            TextField(
                "Bowl Height",
                text: $bowlHeight
            )

            TextField(
                "Chamber Diameter",
                text: $chamberDiameter
            )

            TextField(
                "Chamber Depth",
                text: $chamberDepth
            )
        }
    }

    private var accessoryDimensionsSection: some View {
        VStack(
            alignment: .leading,
            spacing: 12
        ) {
            Text("\(productType.displayName) Dimensions")
                .font(.headline)

            TextField(
                "Length",
                text: $accessoryLength
            )

            TextField(
                "Width",
                text: $accessoryWidth
            )

            TextField(
                "Height",
                text: $accessoryHeight
            )
        }
    }

    private var packageSection: some View {
        VStack(
            alignment: .leading,
            spacing: 12
        ) {
            Text("Package")
                .font(.headline)

            TextField(
                "Length",
                text: $packageLength
            )

            TextField(
                "Width",
                text: $packageWidth
            )

            TextField(
                "Height",
                text: $packageHeight
            )

            TextField(
                "Weight (oz)",
                text: $packageWeight
            )
        }
    }

    private var photosSection: some View {
        VStack(
            alignment: .leading,
            spacing: 12
        ) {
            HStack {
                Text("Photos")
                    .font(.headline)

                Spacer()

                Button("Add Photos") {
                    showingImageImporter = true
                }
            }

            if imageURLs.isEmpty {
                Text("No photos selected")
                    .foregroundStyle(.secondary)
            } else {
                ForEach(
                    Array(imageURLs.enumerated()),
                    id: \.offset
                ) { index, url in
                    HStack {
                        Text("\(index + 1).")

                        Text(url.lastPathComponent)

                        Spacer()

                        Button("Remove") {
                            imageURLs.remove(
                                at: index
                            )
                        }
                    }
                }
            }
        }
    }

    private func createListing() {
        guard
            let price = Double(price),
            let packageLength = Double(packageLength),
            let packageWidth = Double(packageWidth),
            let packageHeight = Double(packageHeight),
            let packageWeight = Double(packageWeight)
        else {
            statusMessage = "Check your numeric fields."
            return
        }

        let productDimensions: ProductDimensions

        switch productType {
        case .pipe:
            guard
                let overallLength = Double(overallLength),
                let bowlHeight = Double(bowlHeight),
                let chamberDiameter = Double(chamberDiameter),
                let chamberDepth = Double(chamberDepth)
            else {
                statusMessage = "Check your pipe dimensions."
                return
            }

            productDimensions = .pipe(
                PipeDimensions(
                    overallLength: overallLength,
                    bowlHeight: bowlHeight,
                    chamberDiameter: chamberDiameter,
                    chamberDepth: chamberDepth
                )
            )

        case .tamper, .ashtray:
            guard
                let length = Double(accessoryLength),
                let width = Double(accessoryWidth),
                let height = Double(accessoryHeight)
            else {
                statusMessage =
                    "Check your \(productType.displayName.lowercased()) dimensions."

                return
            }

            productDimensions = .accessory(
                AccessoryDimensions(
                    length: length,
                    width: width,
                    height: height
                )
            )
        }

        let materials = materials
            .split(separator: ",")
            .map {
                $0.trimmingCharacters(
                    in: .whitespacesAndNewlines
                )
            }
            .filter {
                !$0.isEmpty
            }

        let listing = CreateListingRequest(
            productType: productType,
            title: title,
            price: price,
            description: description,
            materials: materials,
            productDimensions: productDimensions,
            packageDimensions: PackageDimensions(
                length: packageLength,
                width: packageWidth,
                height: packageHeight,
                weightOz: packageWeight
            )
        )

        isUploading = true
        statusMessage = "Creating Etsy draft..."

        Task {
            do {
                let response =
                    try await listingService.createListing(
                        listing: listing,
                        imageURLs: imageURLs
                    )

                await MainActor.run {
                    isUploading = false

                    statusMessage =
                        """
                        Draft created successfully.

                        Listing ID: \(response.listingId ?? 0)
                        State: \(response.state ?? "unknown")
                        Photos: \(response.uploadedImages ?? 0)
                        """
                }
            } catch {
                await MainActor.run {
                    isUploading = false

                    statusMessage =
                        "Failed: \(error.localizedDescription)"
                }
            }
        }
    }
}


