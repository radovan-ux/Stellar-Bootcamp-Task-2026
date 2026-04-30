# Stellaroid Earn

**On-chain credential verification and automated rewards system on Stellar**

## Problem & Solution

**Problem:** A graduating student in the Philippines cannot easily prove their credentials to employers or access financial opportunities, forcing them to rely on manual verification that delays hiring and limits income.

**Solution:** Using Stellar, Stellaroid Earn builds a transparent on-chain system where each certificate has a unique, traceable identity anchored to its rightful owner. Students unlock XLM-based rewards, job payouts, and financial access upon instant credential verification.

## Suggested Timeline for MVP Delivery

- **Week 1:** Smart contract development and testing
- **Week 2:** Frontend integration and wallet setup
- **Week 3:** Deployment to testnet and user testing
- **Week 4:** Documentation and demo preparation

## Stellar Features Used

- **Soroban Smart Contracts**: Core credential registry, tamper-detection, reward, and payment logic
- **XLM Transfers**: Student rewards and employer payouts
- **Custom Tokens**: Optional school-issued credential assets
- **Trustlines**: Credential asset ownership management

## Vision and Purpose

Stellaroid Earn aims to revolutionize credential verification in Southeast Asia by creating a tamper-proof, instantly verifiable system that connects students directly with employment opportunities and financial rewards.

## Prerequisites

- **Rust Toolchain**: Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
