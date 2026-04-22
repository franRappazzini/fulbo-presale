import { BN } from "bn.js";

export function bn(n: number | bigint) {
  return new BN(n);
}
