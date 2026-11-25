import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Metadata } from "next";
import { CodeBlock } from "../../components/CodeBlock";
import { Figure } from "../../components/Figure";
import { Section } from "../../components/Section";
import { SegmentedControl } from "../../components/SegmentedControl";
import { Table } from "../../components/Table";
import styles from "./design.module.css";

export const metadata: Metadata = {
  title: "Design · horsegirl",
  description: "The tokens and base components of the horsegirl website.",
};

// The build runs in site/, and the reference file lives at the repository root.
const checkout = readFileSync(join(process.cwd(), "..", "examples", "checkout.horse"), "utf8");

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
        <Figure caption="Drawn by hand to show the diagram tokens. horsegirl did not draw it.">
          <svg
            viewBox="0 0 520 90"
            width="520"
            height="90"
            role="img"
            aria-label="Web app links to API gateway, which links to Payments"
          >
            <defs>
              <marker
                id="arrow"
                viewBox="0 0 10 10"
                refX="9"
                refY="5"
                markerWidth="7"
                markerHeight="7"
                orient="auto"
              >
                <path d="M0 0L10 5L0 10z" fill="var(--d-ink)" />
              </marker>
            </defs>
            <g className={styles.diagram}>
              <rect x="10" y="25" width="120" height="40" rx="6" />
              <rect x="200" y="25" width="120" height="40" rx="6" className={styles.hot} />
              <rect x="390" y="25" width="120" height="40" rx="6" />
              <path d="M130 45H198" markerEnd="url(#arrow)" />
              <path d="M320 45H388" markerEnd="url(#arrow)" />
              <text x="70" y="50">
                Web app
              </text>
              <text x="260" y="50" className={styles.hotText}>
                API gateway
              </text>
              <text x="450" y="50">
                Payments
              </text>
            </g>
          </svg>
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
