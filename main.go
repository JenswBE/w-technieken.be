package main

import (
	"context"
	"log"
	"os"
	"path"

	"github.com/JenswBE/w-technieken.be/templates"
)

func main() {
	// Output path.
	rootPath := "output-go"
	if err := os.MkdirAll(rootPath, 0755); err != nil {
		log.Fatalf("failed to create output directory: %v", err)
	}

	// Define data
	navLinkStart := templates.NavLink{
		Name: "Start",
		URL:  "/",
	}
	navLinkRealisaties := templates.NavLink{
		Name:     "Realisaties",
		URL:      "/realisaties",
		Children: []templates.NavLink{}, // TODO
	}
	navLinkOverOns := templates.NavLink{
		Name: "Over ons",
		URL:  "/over-ons",
	}
	navLinkOnzeDiensten := templates.NavLink{
		Name: "Onze diensten",
		URL:  "/onze-diensten",
	}
	baseCommon := templates.TemplateBaseCommon{
		NavLinks: []templates.NavLink{
			navLinkStart,
			navLinkRealisaties,
			navLinkOverOns,
			navLinkOnzeDiensten,
		},
	}
	baseSecific := templates.TemplateBaseSpecific{
		CanonicalURL: "https://w-technieken.be/",
		CurrentLink:  navLinkStart,
		Title:        "W-Technieken Bevel - Home",
	}

	// Create an index page.
	name := path.Join(rootPath, "index.html")
	f, err := os.Create(name)
	if err != nil {
		log.Fatalf("failed to create output file: %v", err)
	}

	// Write it out.
	err = templates.Base(baseCommon, baseSecific, templates.Empty(), templates.Empty()).Render(context.Background(), f)
	if err != nil {
		log.Fatalf("failed to write index page: %v", err)
	}
}
