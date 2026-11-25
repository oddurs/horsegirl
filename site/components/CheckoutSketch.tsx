import styles from "./CheckoutSketch.module.css";

// The flow of examples/checkout.horse, drawn by hand from the positions on the
// concept page. horsegirl cannot draw yet, so the caption beside this picture
// must say who drew it.
const W = 110;
const H = 40;

const things: [string, number, number][] = [
  ["Web app", 20, 40],
  ["Mobile app", 20, 110],
  ["API gateway", 180, 75],
  ["Orders", 415, 50],
  ["Payments", 415, 120],
  ["Notify", 415, 190],
  ["Postgres", 415, 300],
  ["Events", 180, 300],
  ["Email provider", 20, 300],
];

// Each link: its path, then its label and where the label sits.
const links: [string, string?, number?, number?, ("start" | "middle" | "end")?][] = [
  ["M130 60H155V88H180", "POST /checkout", 136, 52, "start"],
  ["M130 130H155V102H180"],
  ["M290 88H388V70H415", "create", 300, 82, "start"],
  ["M290 102H388V140H415"],
  ["M470 90V120", "charge", 477, 109, "start"],
  ["M470 160V190", "paid", 477, 179, "start"],
  ["M525 70H555V320H525", "save", 549, 282, "end"],
  ["M415 210H330V320H290", "publish", 337, 286, "start"],
  ["M180 320H130", "send", 155, 313, "middle"],
];

export function CheckoutSketch() {
  return (
    <svg
      className={styles.sketch}
      viewBox="0 0 600 360"
      width="600"
      height="360"
      role="img"
      aria-label="A flow of the checkout file. Web app and Mobile app link to API gateway. API gateway links to Orders and Payments, inside a Services group with Notify. Orders charges Payments and saves to Postgres. Payments tells Notify, Notify publishes to Events, and Events sends to Email provider."
    >
      <defs>
        <marker
          id="checkout-arrow"
          viewBox="0 0 10 10"
          refX="9"
          refY="5"
          markerWidth="7"
          markerHeight="7"
          orient="auto"
        >
          <path d="M0 0L10 5L0 10z" className={styles.head} />
        </marker>
      </defs>
      <rect className={styles.group} x="375" y="20" width="190" height="230" rx="10" />
      <text className={styles.label} x="387" y="38">
        Services
      </text>
      {links.map(([d, label, x, y, anchor]) => (
        <g key={d}>
          <path className={styles.link} d={d} markerEnd="url(#checkout-arrow)" />
          {label && (
            <text className={styles.label} x={x} y={y} textAnchor={anchor}>
              {label}
            </text>
          )}
        </g>
      ))}
      {things.map(([name, x, y]) => (
        <g key={name}>
          <rect className={styles.thing} x={x} y={y} width={W} height={H} rx="6" />
          <text className={styles.name} x={x + W / 2} y={y + 24} textAnchor="middle">
            {name}
          </text>
        </g>
      ))}
    </svg>
  );
}
