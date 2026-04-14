import { bn } from "./utils/functions";

/*
    | Stage | Target USD | Price | Tokens Sold | Locked % | Unlock |
    | --- | --- | --- | --- | --- | --- |
    | 1 | 10,000 | 0.000500 | 20,000,000 | 50% | 5% / month |
    | 2 | 20,000 | 0.000700 | 28,571,429 | 50% | 5% / month |
    | 3 | 30,000 | 0.000900 | 33,333,333 | 50% | 5% / month |
    | 4 | 40,000 | 0.001100 | 36,363,636 | 50% | 5% / month |
    | 5 | 50,000 | 0.001400 | 35,714,286 | 35% | 5% / month |
    | 6 | 60,000 | 0.001800 | 33,333,333 | 35% | 5% / month |
    | 7 | 70,000 | 0.002300 | 30,434,783 | 35% | 5% / month |
    | 8 | 80,000 | 0.002900 | 27,586,207 | 35% | 5% / month |
    | 9 | 90,000 | 0.003700 | 24,324,324 | 20% | 5% / month |
    | 10 | 100,000 | 0.005000 | 20,000,000 | 20% | 5% / month |
    | 11 | 110,000 | 0.006500 | 10,338,669 | 20% | 5% / month |
*/

/*  priceUsd: u64;
  maxTokens: u64;
  tokensSold: u64;
  raisedSol: u64;
  lockedPctBps: u16;
  maxWalletPctBps: u16; */
export const stages = [
  {
    priceUsd: bn(500),
    maxTokens: bn(20000000),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 5000,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(700),
    maxTokens: bn(28571429),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 5000,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(900),
    maxTokens: bn(33333333),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 5000,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(1100),
    maxTokens: bn(36363636),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 5000,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(1400),
    maxTokens: bn(35714286),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 3500,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(1800),
    maxTokens: bn(33333333),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 3500,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(2300),
    maxTokens: bn(30434783),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 3500,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(2900),
    maxTokens: bn(27586207),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 3500,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(3700),
    maxTokens: bn(24324324),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 2000,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(5000),
    maxTokens: bn(20000000),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 2000,
    maxWalletPctBps: 5000,
  },
  {
    priceUsd: bn(6500),
    maxTokens: bn(10338669),
    tokensSold: bn(0),
    raisedSol: bn(0),
    lockedPctBps: 2000,
    maxWalletPctBps: 5000,
  },
];
