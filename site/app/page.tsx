import { readFileSync } from "node:fs";
import { join } from "node:path";
import Link from "next/link";
import { CodeBlock } from "../components/CodeBlock";
import { Section } from "../components/Section";
import { binaryPath, draw, root } from "../lib/gallery.ts";
import { highlightHorsegirl } from "../lib/highlight.ts";
import { readCommands, readSpec } from "../lib/status.ts";
import styles from "./home.module.css";

const roadmap = "https://github.com/oddurs/horsegirl/blob/main/ROADMAP.md";

// The build runs in site/, and the reference file lives at the repository root.
const checkout = readFileSync(join(process.cwd(), "..", "examples", "checkout.horse"), "utf8");

const checkoutSvg = draw(binaryPath(), join(root, "examples", "checkout.horse")).svg;

// Each statement says what horsegirl is built to do. The status section says
// what of it works today.
const statements = [
  {
    who: "For the agent",
    what: "Five rules to learn, on one page. The check will name the fix for each error, and the tool will describe each picture as text.",
  },
  {
    who: "For the reader",
    what: "Pictures will be plain by default and keep their shape when one line of the source changes.",
  },
  {
    who: "For the repository",
    what: "One file of facts will draw many pictures, and its rules will fail the build.",
  },
];

const rules = [
  {
    rule: "A name is a thing.",
    detail: "Names can have spaces. A thing exists when it is first mentioned.",
    code: "Web app",
  },
  {
    rule: "An arrow is a link.",
    detail:
      "Commas fan out. Words after the colon label the link. -x is a link that must not exist. A final ? marks a guess.",
    code: "Web app, Mobile app -> API gateway: HTTPS\nWeb app -x Postgres: never directly\nOrders -> Cache ?",
  },
  {
    rule: "Braces hold what is inside and what is known.",
    detail: "A bare name inside braces is a part. A key: value pair is a fact.",
    code: "Services { Orders, Payments, Notify }\nPayments { owner: Commerce }",
  },
  {
    rule: "A story is links in the order they happen.",
    detail:
      "Outside a story, order is free. Inside a story, the order you write is the order of events.",
    code: "story checkout {\n  Orders -> Payments: charge\n  Payments -> Notify: paid\n}",
  },
  {
    rule: "show picks a lens.",
    detail: "A lens says what position means. The same facts feed every lens.",
    code: "show flow\nshow sequence of checkout\nshow rings from Payments",
  },
];

function Source({ text }: { text: string }) {
  // biome-ignore lint/security/noDangerouslySetInnerHtml: highlightHorsegirl escapes every character of the source.
  return <code dangerouslySetInnerHTML={{ __html: highlightHorsegirl(text) }} />;
}

export default function Home() {
  const commands = readCommands();
  return (
    <>
      <section className={styles.hero} aria-labelledby="top-title">
        <p className={styles.status}>
          Not released. A build from source prints the language card.{" "}
          <a href="#status">What works today</a>
        </p>
        <h1 id="top-title">Write what is true. Choose how to look.</h1>
        <p className={styles.lede}>
          horsegirl is a diagram language for the case where an agent writes the diagram and a
          person reads it. A file states the facts about a system once. A lens will turn the same
          facts into a flow, a sequence, a matrix or another picture.
        </p>

        <div className={styles.window}>
          <div className={styles.bar}>
            <span className={styles.file}>examples/checkout.horse</span>
            <span className={styles.lens}>show flow</span>
          </div>
          <div className={styles.split}>
            {/* biome-ignore lint/a11y/noNoninteractiveTabindex: a scrolling region must take focus. */}
            <pre className={styles.source} tabIndex={0}>
              <Source text={checkout.trimEnd()} />
            </pre>
            <figure className={styles.picture}>
              {/* biome-ignore lint/security/noDangerouslySetInnerHtml: the SVG is the output of our own program, drawn from a file of this repository. */}
              <div dangerouslySetInnerHTML={{ __html: checkoutSvg }} />
              <figcaption>
                Drawn when this site was built, by <code>horsegirl draw checkout.horse</code>. See
                the <Link href="/gallery/">gallery</Link> for more.
              </figcaption>
            </figure>
          </div>
          <div className={styles.out}>
            <p className={styles.prompt}>$ horsegirl spec</p>
            {/* biome-ignore lint/a11y/noNoninteractiveTabindex: a scrolling region must take focus. */}
            <pre className={styles.card} tabIndex={0}>
              {readSpec().trimEnd()}
            </pre>
          </div>
        </div>
        <p className={styles.note}>
          The file and the card are real: the file is the reference file in the repository, and the
          card is what <code>horsegirl spec</code> prints today.
        </p>

        <ul className={styles.props}>
          {statements.map(({ who, what }) => (
            <li key={who}>
              <strong>{who}</strong>
              <span>{what}</span>
            </li>
          ))}
        </ul>
      </section>

      <Section
        id="language"
        eyebrow="The language"
        title="Five rules. That is all of it."
        lede={
          <>
            A person learns the language from this table. An agent learns it from the same table,
            printed by <code>horsegirl spec</code>.
          </>
        }
      >
        <ol className={styles.rules}>
          {rules.map(({ rule, detail, code }) => (
            <li key={rule}>
              <div className={styles.rule}>
                <strong>{rule}</strong>
                <span>{detail}</span>
              </div>
              {/* biome-ignore lint/a11y/noNoninteractiveTabindex: a scrolling region must take focus. */}
              <pre className={styles.example} tabIndex={0}>
                <Source text={code} />
              </pre>
            </li>
          ))}
        </ol>
      </Section>

      <Section
        id="status"
        eyebrow="Status"
        title="What works today"
        lede="There is no release yet. A binary built from source does this and nothing more:"
      >
        <dl className={styles.commands}>
          {commands.map(({ name, does }) => (
            <div key={name}>
              <dt>
                <code>horsegirl {name}</code>
              </dt>
              <dd>{does}</dd>
            </div>
          ))}
        </dl>
        <p>
          The parser, <code>check</code>, <code>fix</code>, the flow lens drawn as SVG and the MCP
          server are the work of the first release, v0.1. The <a href={roadmap}>roadmap</a> lists
          every item and what it is waiting for.
        </p>
        <CodeBlock file="Build from source">
          {
            "git clone https://github.com/oddurs/horsegirl\ncd horsegirl\ncargo install --path crates/horsegirl --locked\nhorsegirl spec"
          }
        </CodeBlock>
        <p>
          The <Link href="/docs/">documentation</Link> holds the design.
        </p>
      </Section>
    </>
  );
}
