import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Shell } from "../components/Shell";
import "./fonts.css";
import "./tokens.css";
import "./highlight.css";

export const metadata: Metadata = {
  title: "horsegirl",
  description: "An agent-first diagram language: state the facts once, choose a lens.",
};

const nav = [
  { href: "/docs/", label: "Docs" },
  { href: "/design/", label: "Design" },
  { href: "https://github.com/oddurs/horsegirl", label: "GitHub" },
];

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Shell nav={nav}>{children}</Shell>
      </body>
    </html>
  );
}
