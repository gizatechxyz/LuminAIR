// TypeScript definitions for LuminAIR WASM Verifier
// This file provides type definitions for better TypeScript integration

/**
 * Verification result returned by proof verification functions
 */
export interface VerificationResult {
  readonly success: boolean;
  readonly error_message?: string;
}

/**
 * Initialize the WASM module
 * Must be called before using any verification functions
 */
declare function init(
  module?: WebAssembly.Module | Promise<WebAssembly.Module>
): Promise<void>;
export default init;

/**
 * Verifies a LuminAIR proof from binary data
 * @param proofBytes - Binary proof data as Uint8Array
 * @param settingsBytes - Binary settings data as Uint8Array
 * @returns Verification result
 */
export function verify_proof_wasm(
  proofBytes: Uint8Array,
  settingsBytes: Uint8Array
): VerificationResult;

/**
 * Tests if the WASM module is working correctly
 * @returns Success message
 */
export function test_wasm_module(): string;

/**
 * Returns the version of the verifier
 * @returns Version string
 */
export function get_version(): string;

/**
 * Error types that may be thrown during verification
 */
export interface LuminairVerificationError extends Error {
  name: "LuminairVerificationError";
  message: string;
  cause?: Error;
}

/**
 * Configuration options for the verifier
 */
export interface VerifierConfig {
  /**
   * Enable debug logging to console
   * @default false
   */
  enableDebugLogging?: boolean;

  /**
   * Maximum memory allocation for verification (in MB)
   * @default undefined (no limit)
   */
  maxMemoryMB?: number;
}

/**
 * Advanced verification options
 */
export interface VerificationOptions {
  /**
   * Timeout for verification in milliseconds
   * @default 60000 (60 seconds)
   */
  timeoutMs?: number;

  /**
   * Configuration for the verifier
   */
  config?: VerifierConfig;
}

/**
 * Extended verification result with timing information
 */
export interface DetailedVerificationResult extends VerificationResult {
  /**
   * Time taken for verification in milliseconds
   */
  verificationTimeMs?: number;

  /**
   * Memory usage during verification in MB
   */
  memoryUsageMB?: number;
}

/**
 * Utility functions for working with LuminAIR proofs
 */
export declare namespace LuminairUtils {
  /**
   * Validates that binary data contains a valid proof structure
   * @param proofBytes - Binary data to validate
   * @returns true if valid, false otherwise
   */
  function isValidProofBinary(proofBytes: Uint8Array): boolean;

  /**
   * Validates that binary data contains valid settings structure
   * @param settingsBytes - Binary data to validate
   * @returns true if valid, false otherwise
   */
  function isValidSettingsBinary(settingsBytes: Uint8Array): boolean;

  /**
   * Estimates the size of a proof in bytes
   * @param proofBytes - Binary proof data
   * @returns Size in bytes
   */
  function getProofSize(proofBytes: Uint8Array): number;
}
