import type { Metadata } from "next";
import { CodeBlock } from "../../components/CodeBlock";
import { Section } from "../../components/Section";
import { drawGallery } from "../../lib/gallery.ts";
import { highlightHorsegirl } from "../../lib/highlight.ts";
import styles from "./gallery.module.css";

export const metadata: Metadata = {
  title: "Gallery · horsegirl",
  description: "Pictures that the horsegirl program drew from the files beside them.",
};

function Source({ text }: { text: string }) {
  // biome-ignore lint/security/noDangerouslySetInnerHtml: highlightHorsegirl escapes every character of the source.
  return <span dangerouslySetInnerHTML={{ __html: highlightHorsegirl(text) }} />;
}

export default function Gallery() {
  const pictures = drawGallery();
  return (
    <>
      <div className={styles.intro}>
        <h1>Gallery</h1>
        <p className={styles.lede}>
          Each picture here was drawn when this site was built. The build ran the command under it
          on the file beside it, with the horsegirl program of the same commit. No picture is drawn
          by hand or kept as a file. All use the <code>flow</code> lens.
        </p>
      </div>
      {pictures.map((picture) => (
        <Section key={picture.name} id={picture.name.replace(/\.horse$/, "")} title={picture.name}>
          <div className={styles.pair}>
            {/* biome-ignore lint/security/noDangerouslySetInnerHtml: the SVG is the output of our own program, drawn from a file of this repository. */}
            <div className={styles.canvas} dangerouslySetInnerHTML={{ __html: picture.svg }} />
            <CodeBlock file={picture.command}>
              <Source text={picture.source} />
            </CodeBlock>
          </div>
        </Section>
      ))}
    </>
  );
}
