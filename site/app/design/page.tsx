import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Metadata } from "next";
import { CodeBlock } from "../../components/CodeBlock";
import { Figure } from "../../components/Figure";
import { Section } from "../../components/Section";
import { SegmentedControl } from "../../components/SegmentedControl";
import { Table } from "../../components/Table";
import { binaryPath, draw, root } from "../../lib/gallery.ts";
import styles from "./design.module.css";

export const metadata: Metadata = {
  title: "Design · horsegirl",
  description: "The tokens and base components of the horsegirl website.",
};

// The build runs in site/, and the reference file lives at the repository root.
const checkout = readFileSync(join(process.cwd(), "..", "examples", "checkout.horse"), "utf8");

const drawn = draw(binaryPath(), join(root, "examples", "checkout.horse"));

const page = ["ground", "surface", "ink", "muted", "line", "accent", "accent-ink"];
const meaning = ["turf", "saddle", "t-pink", "t-green", "t-ochre", "t-neutral"];
const diagram = [
  "d-paper",
  "d-ink",
  "d-muted",
  "d-line",
  "d-accent",
  "d-c0",
  "d-c1",
  "d-c2",
  "d-c3",
];
const space = ["s1", "s2", "s3", "s4", "s5", "s6"];
const radii = ["r1", "r2", "r3"];

const lenses = [
  { lens: "flow", dials: "x: depth, inside: group", solver: "layered" },
  { lens: "sequence", dials: "x: thing, y: step", solver: "grid" },
  { lens: "lanes", dials: "x: step, y: owner", solver: "grid" },
  { lens: "matrix", dials: "x: to, y: from", solver: "grid" },
];

function Swatches({ names }: { names: string[] }) {
  return (
    <ul className={styles.swatches}>
      {names.map((name) => (
        <li key={name}>
          <span className={styles.chip} style={{ background: `var(--${name})` }} />
          <code>--{name}</code>
        </li>
      ))}
    </ul>
  );
}

export default function Design() {
  return (
    <>
      <div className={styles.intro}>
        <h1>Design</h1>
        <p className={styles.lede}>
          The tokens and base components of this site. They come from the concept page in{" "}
          <code>docs/concept/index.html</code>. Every value has a light and a dark form.
        </p>
      </div>

      <Section
        id="colour"
        eyebrow="Tokens"
        title="Colour"
        lede="A neutral page and one brand colour. Turf and saddle name diagram categories."
      >
        <Swatches names={page} />
        <Swatches names={meaning} />
      </Section>

      <Section
        id="diagram"
        eyebrow="Tokens"
        title="Diagram colour"
        lede="A picture reads only the --d-* tokens, so a diagram theme never touches the page."
      >
        <Swatches names={diagram} />
      </Section>

      <Section id="space" eyebrow="Tokens" title="Space and radius">
        <ul className={styles.scale}>
          {space.map((name) => (
            <li key={name}>
              <code>--{name}</code>
              <span className={styles.bar} style={{ width: `var(--${name})` }} />
            </li>
          ))}
        </ul>
        <ul className={styles.radii}>
          {radii.map((name) => (
            <li key={name} style={{ borderRadius: `var(--${name})` }}>
              <code>--{name}</code>
            </li>
          ))}
        </ul>
      </Section>

      <Section id="type" eyebrow="Tokens" title="Type">
        <div className={styles.type}>
          <h3>A heading of the third level</h3>
          <p>
            Body text is set in the body face at 16 pixels. A name such as <code>API gateway</code>{" "}
            is set in the mono face.
          </p>
        </div>
      </Section>

      <Section
        id="components"
        eyebrow="Components"
        title="Base components"
        lede="The page shell and the sticky header frame this page. The others follow."
      >
        <SegmentedControl
          label="Lens"
          defaultValue="flow"
          options={lenses.map(({ lens }) => ({ value: lens, label: lens }))}
        />
        <CodeBlock file="checkout.horse">{checkout}</CodeBlock>
        <Figure caption={`Drawn by ${drawn.command}, when this site was built.`}>
          {/* biome-ignore lint/security/noDangerouslySetInnerHtml: the SVG is the output of our own program, drawn from a file of this repository. */}
          <div dangerouslySetInnerHTML={{ __html: drawn.svg }} />
        </Figure>
        <Table caption="Four lenses and the dials they bind">
          <thead>
            <tr>
              <th scope="col">Lens</th>
              <th scope="col">Dials</th>
              <th scope="col">Solver</th>
            </tr>
          </thead>
          <tbody>
            {lenses.map(({ lens, dials, solver }) => (
              <tr key={lens}>
                <td>{lens}</td>
                <td>
                  <code>{dials}</code>
                </td>
                <td>{solver}</td>
              </tr>
            ))}
          </tbody>
        </Table>
      </Section>
    </>
  );
}
