import type { Metadata } from "next";

// Separate file for home page metadata since page.tsx is a client component
export const homeMetadata: Metadata = {
  title: "Hüma — Code as You Think",
  description:
    "Experimental programming language with Turkish-oriented syntax and a Rust-based interpreter.",
};
