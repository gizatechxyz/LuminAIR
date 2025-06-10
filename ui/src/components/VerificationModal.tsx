"use client";

import React from "react";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { Check, Loader2, X, Download } from "lucide-react";
import { cn } from "../lib/utils";
import JSZip from "jszip";
import { GraphVisualizer } from "./GraphVisualizer";

// Verification steps based on the actual console logs from WASM Verifier
const VERIFICATION_STEPS = [
  {
    id: "setup",
    title: "Protocol Setup",
    description: "Initializing verifier components",
    patterns: [
      "🚀 Starting LuminAIR proof verification",
      "⚙️ Protocol Setup: Initializing verifier components",
      "✅ Protocol Setup: Configuration complete",
    ],
  },
  {
    id: "preprocessed",
    title: "Commit preprocessed trace",
    description: "Processing preprocessed trace commitments",
    patterns: [
      "🔄 Interaction Phase 0: Processing preprocessed trace",
      "✅ Interaction Phase 0: Preprocessed trace committed",
    ],
  },
  {
    id: "main",
    title: "Commit main trace",
    description: "Processing main execution trace",
    patterns: [
      "🔄 Interaction Phase 1: Processing main trace",
      "✅ Interaction Phase 1: Main trace committed",
    ],
  },
  {
    id: "interaction",
    title: "Commit interaction trace",
    description: "Processing interaction trace commitments",
    patterns: [
      "🔄 Interaction Phase 2: Processing interaction trace",
      "✅ Interaction Phase 2: Interaction trace committed",
    ],
  },
  {
    id: "verify",
    title: "Verify proof with STWO",
    description: "Verifying STARK proof with STWO prover",
    patterns: [
      "🔍 Proof Verification: Verifying STARK proof",
      "✅ Proof Verification: STARK proof is valid",
    ],
  },
];

export type StepStatus = "pending" | "in-progress" | "completed" | "error";

export interface StepState {
  status: StepStatus;
  message?: string;
}

export interface VerificationState {
  isOpen: boolean;
  isVerifying: boolean;
  steps: Record<string, StepState>;
  allStepsCompleted: boolean;
  result?: {
    success: boolean;
    message?: string;
  };
}

interface VerificationModalProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Function to handle modal close */
  onOpenChange: (open: boolean) => void;
  /** Current verification state */
  verificationState: VerificationState;
  /** Path to the proof file */
  proofPath: string;
  /** Path to the settings file */
  settingsPath: string;
  /** Path to the graph visualization file */
  graphPath: string;
  /** Title displayed in the modal (default: "Can't be evil.") */
  title?: string;
  /** Author name displayed in the modal (default: "Giza") */
  author?: string;
  /** Model description displayed in the modal (default: "Demo model") */
  modelDescription?: string;
  /** Author URL (default: "https://www.gizatech.xyz/") */
  authorUrl?: string;
}

export function VerificationModal({
  isOpen,
  onOpenChange,
  verificationState,
  proofPath,
  settingsPath,
  graphPath,
  title = "Can't be evil.",
  author = "Giza",
  modelDescription = "Demo model",
  authorUrl = "https://www.gizatech.xyz/",
}: VerificationModalProps) {
  const getOverallStatus = () => {
    if (verificationState.result && !verificationState.result.success) {
      return "error";
    }

    if (verificationState.result && verificationState.result.success && verificationState.allStepsCompleted) {
      return "completed";
    }

    if (
      verificationState.isVerifying ||
      (verificationState.result && verificationState.result.success && !verificationState.allStepsCompleted)
    ) {
      return "in-progress";
    }

    return "pending";
  };



  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[1200px] max-h-[90vh] font-sans bg-white dark:bg-black border-gray-200 dark:border-gray-700">
        <DialogHeader className="pb-3">
          <DialogTitle className="text-2xl font-bold text-left mb-2 text-gray-900 dark:text-gray-100">
            {title}
          </DialogTitle>
        </DialogHeader>

        <div className="flex flex-col lg:flex-row gap-6 min-h-[500px] max-h-[75vh] overflow-auto">
          {/* Left column */}
          <div className="flex-1 flex flex-col space-y-4 order-2 lg:order-1">
            {/* Graph Visualizer */}
            <div className="flex-1 min-h-0">
              <GraphVisualizer
                graphPath={graphPath}
                className="h-full"
              />
            </div>

            {/* Bottom section with download button and footer */}
            <div className="flex flex-col space-y-3 mt-auto">
              {/* Download button */}
              {verificationState.result && (
                <Button
                  variant="outline"
                  size="sm"
                  className="text-xs font-mono border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 self-start px-3 py-1.5 h-auto"
                  onClick={async () => {
                    try {
                      const zip = new JSZip();

                      const [proofResp, settingsResp] = await Promise.all([
                        fetch(proofPath),
                        fetch(settingsPath),
                      ]);

                      if (proofResp.ok) {
                        const proofBlob = await proofResp.blob();
                        zip.file("proof.bin", proofBlob);
                      }

                      if (settingsResp.ok) {
                        const settingsBlob = await settingsResp.blob();
                        zip.file("settings.bin", settingsBlob);
                      }

                      const zipBlob = await zip.generateAsync({
                        type: "blob",
                      });
                      const url = URL.createObjectURL(zipBlob);
                      const a = document.createElement("a");
                      a.href = url;
                      a.download = "luminair-proof.zip";
                      document.body.appendChild(a);
                      a.click();
                      document.body.removeChild(a);
                      URL.revokeObjectURL(url);
                    } catch (error) {
                      console.error("Download failed:", error);
                    }
                  }}
                >
                  <Download className="h-3 w-3 mr-1.5" />
                  Download proof
                </Button>
              )}

              {/* Footer information */}
              <div className="space-y-2">
                <div className="flex items-center space-x-1 text-xs text-gray-400 dark:text-gray-500">
                  <svg
                    className="h-3 w-3"
                    viewBox="0 0 18 20"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path
                      fillRule="evenodd"
                      clipRule="evenodd"
                      d="M8.65331 0L0 14.9659L8.65331 20L17.3132 14.9659L8.65331 0ZM7.2009 8.60339C8.12395 7.67922 8.6235 6.33476 8.65835 4.68359C8.72707 7.93945 10.6026 10.0027 13.9692 10.0027C12.3099 10.0027 11.0129 10.5039 10.1158 11.4021C9.19275 12.3263 8.6932 13.6707 8.65835 15.3219C8.58963 12.066 6.71409 10.0027 3.34753 10.0027C5.00678 10.0027 6.30384 9.50154 7.2009 8.60339Z"
                      fill="currentColor"
                    />
                  </svg>
                  <span>Made By Giza</span>
                </div>
              </div>
            </div>
          </div>

          {/* Right column - Description, Terminal, and Status */}
          <div className="flex-1 flex flex-col space-y-4 order-1 lg:order-2">
            {/* Description */}
            <DialogDescription className="text-sm text-gray-600 dark:text-gray-300">
              This verification occurs entirely in your browser,
              cryptographically verifying that the computational graph
              executed precisely as intended. By leveraging zero-knowledge proofs,
              we mathematically guarantee the integrity of the computation.
            </DialogDescription>

            {/* Terminal container */}
            <div className="bg-white dark:bg-gray-950 rounded-lg p-3 font-mono text-xs border border-gray-200 dark:border-gray-700 h-56 flex flex-col">
              <div className="flex items-center mb-2 pb-2 border-b border-gray-200 dark:border-gray-700">
                <div className="text-gray-400 text-xs">verification logs</div>
              </div>

              <div className="space-y-1 flex-1 overflow-y-auto">
                {VERIFICATION_STEPS.map((step, index) => {
                  const stepState = verificationState.steps[step.id];
                  const isActive = stepState.status === "in-progress";

                  return (
                    <div
                      key={step.id}
                      className={cn(
                        "flex items-center space-x-2 py-0.5 transition-all",
                        isActive && "animate-pulse",
                        stepState.status === "pending" &&
                          !verificationState.isVerifying &&
                          "text-gray-600"
                      )}
                    >
                      {stepState.status === "pending" ? (
                        <span className="text-gray-600">○</span>
                      ) : stepState.status === "in-progress" ? (
                        <Loader2 className="h-3 w-3 animate-spin text-yellow-400" />
                      ) : stepState.status === "completed" ? (
                        <span className="text-gray-500">✓</span>
                      ) : (
                        <span className="text-red-400">✗</span>
                      )}

                      <span
                        className={cn(
                          "text-xs",
                          stepState.status === "pending" &&
                            "text-gray-700 dark:text-gray-400",
                          stepState.status === "in-progress" &&
                            "text-gray-600 dark:text-gray-300",
                          stepState.status === "completed" &&
                            "text-gray-500 dark:text-gray-400",
                          stepState.status === "error" && "text-red-400"
                        )}
                      >
                        {step.title}
                      </span>
                    </div>
                  );
                })}

                {verificationState.isVerifying && (
                  <div className="flex items-center space-x-2 py-0.5">
                    <Loader2 className="h-3 w-3 animate-spin text-blue-400" />
                    <span className="text-blue-400 text-xs">
                      Processing verification...
                    </span>
                  </div>
                )}

                {(getOverallStatus() === "completed" ||
                  (verificationState.result && !verificationState.result.success)) && (
                  <div className="flex items-center space-x-2 py-0.5 mt-2 pt-2 border-t border-gray-200 dark:border-gray-700">
                    <span
                      className={cn(
                        verificationState.result?.success
                          ? "text-green-400"
                          : "text-red-400"
                      )}
                    >
                      {verificationState.result?.success ? "✓" : "✗"}
                    </span>
                    <span
                      className={cn(
                        "text-xs",
                        verificationState.result?.success
                          ? "text-green-400"
                          : "text-red-400"
                      )}
                    >
                      {verificationState.result?.success
                        ? "Verification completed successfully"
                        : `Verification failed: ${verificationState.result?.message}`}
                    </span>
                  </div>
                )}
              </div>
            </div>

            <div className="flex-1"></div>

            {/* Additional fields */}
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                  Status:
                </span>
                <div
                  className={cn(
                    "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium",
                    getOverallStatus() === "completed" &&
                      "bg-green-100 dark:bg-green-950/50 text-green-800 dark:text-green-300",
                    getOverallStatus() === "in-progress" &&
                      "bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300",
                    getOverallStatus() === "error" &&
                      "bg-red-100 dark:bg-red-950/50 text-red-800 dark:text-red-300",
                    getOverallStatus() === "pending" &&
                      "bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300"
                  )}
                >
                  {getOverallStatus() === "completed" && (
                    <>
                      <Check className="w-3 h-3 mr-1" />
                      Verified
                    </>
                  )}
                  {getOverallStatus() === "in-progress" && (
                    <>
                      <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                      Verifying
                    </>
                  )}
                  {getOverallStatus() === "error" && (
                    <>
                      <X className="w-3 h-3 mr-1" />
                      Failed
                    </>
                  )}
                  {getOverallStatus() === "pending" && (
                    <>
                      <div className="w-3 h-3 mr-1 rounded-full border border-current" />
                      Pending
                    </>
                  )}
                </div>
              </div>

              <div className="flex justify-between items-start">
                <span className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                  Author:
                </span>
                <span className="text-xs text-gray-700 dark:text-gray-300 text-right max-w-[200px]">
                  <a
                    href={authorUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:underline"
                  >
                    {author}
                  </a>
                </span>
              </div>

              <div className="flex justify-between items-start">
                <span className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                  Model description:
                </span>
                <span className="text-xs text-gray-700 dark:text-gray-300 text-right max-w-[200px]">
                  {modelDescription}
                </span>
              </div>

              <div className="flex justify-between items-start">
                <span className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                  Prover:
                </span>
                <span className="text-xs text-gray-700 dark:text-gray-300 text-right max-w-[200px]">
                  <a
                    href="https://github.com/gizatechxyz/LuminAIR"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="hover:underline"
                  >
                    LuminAIR STWO
                  </a>
                </span>
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export { VERIFICATION_STEPS }; 