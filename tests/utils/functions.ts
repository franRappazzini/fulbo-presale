import { BN } from "bn.js";

export function bn(n: number | bigint) {
  return new BN(n);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function logJson(label: string, obj: unknown) {
  console.log(
    label,
    JSON.stringify(obj, (_, v) => (typeof v === "bigint" ? v.toString() : v), 2),
  );
}
