"use client";

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { Check, Loader2, X, AlertCircle } from "lucide-react";
import { cn } from "../lib/utils";
import init, { verify } from "@gizatech/luminair-web";
import JSZip from "jszip";

// Verification steps based on the actual console logs from WASM
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

type StepStatus = "pending" | "in-progress" | "completed" | "error";

interface StepState {
  status: StepStatus;
  message?: string;
}

interface VerificationState {
  isOpen: boolean;
  isVerifying: boolean;
  steps: Record<string, StepState>;
  allStepsCompleted: boolean;
  result?: {
    success: boolean;
    message?: string;
  };
}

export interface VerifyButtonProps {
  /** Path to the proof file (required) */
  proofPath: string;
  /** Path to the settings file (required) */
  settingsPath: string;
  /** Title displayed in the modal (default: "Can't be evil.") */
  title?: string;
  /** Text displayed on the button (default: "VERIFY") */
  buttonText?: string;
  /** Author name displayed in the modal (default: "Giza") */
  author?: string;
  /** Model description displayed in the modal (default: "Demo model") */
  modelDescription?: string;
  /** Author URL (default: "https://www.gizatech.xyz/") */
  authorUrl?: string;
  /** Custom className for the button */
  className?: string;
}

export function VerifyButton({
  proofPath,
  settingsPath,
  title = "Can't be evil.",
  buttonText = "VERIFY",
  author = "Giza",
  modelDescription = "Demo model",
  authorUrl = "https://www.gizatech.xyz/",
  className,
}: VerifyButtonProps) {
  const [state, setState] = useState<VerificationState>({
    isOpen: false,
    isVerifying: false,
    allStepsCompleted: false,
    steps: VERIFICATION_STEPS.reduce(
      (acc, step) => ({
        ...acc,
        [step.id]: { status: "pending" as StepStatus },
      }),
      {}
    ),
  });

  const originalConsoleLog = useRef<any>(null);
  const originalConsoleInfo = useRef<any>(null);
  const stepTimestamps = useRef<Record<string, number>>({});

  const updateStepStatus = useCallback(
    (stepId: string, status: StepStatus, message?: string) => {
      setState((prev) => {
        const newSteps = {
          ...prev.steps,
          [stepId]: { status, message },
        };

        // Check if all steps are completed
        const allCompleted = VERIFICATION_STEPS.every(
          (step) => newSteps[step.id]?.status === "completed"
        );

        return {
          ...prev,
          steps: newSteps,
          allStepsCompleted: allCompleted,
        };
      });
    },
    []
  );

  const updateStepWithDelay = useCallback(
    async (stepId: string, status: StepStatus, message?: string) => {
      const now = Date.now();
      const lastUpdate = stepTimestamps.current[stepId] || 0;

      // Get step index for incremental delay
      const stepIndex = VERIFICATION_STEPS.findIndex(
        (step) => step.id === stepId
      );
      const baseDelay = 200; // Base delay of 200ms
      const incrementDelay = 50; // Add 50ms for each step
      const minDelay = baseDelay + stepIndex * incrementDelay; // 200ms, 250ms, 300ms, 350ms, 400ms

      const elapsed = now - lastUpdate;

      if (elapsed < minDelay) {
        await new Promise((resolve) => setTimeout(resolve, minDelay - elapsed));
      }

      stepTimestamps.current[stepId] = Date.now();
      updateStepStatus(stepId, status, message);
    },
    [updateStepStatus]
  );

  // Monitor console logs for verification steps
  useEffect(() => {
    if (!state.isVerifying) return;

    // Reset timestamps when starting verification
    stepTimestamps.current = {};

    // Store original console methods
    originalConsoleLog.current = console.log;
    originalConsoleInfo.current = console.info;

    const handleLogMessage = (message: string) => {
      // Check each verification step for matching patterns
      VERIFICATION_STEPS.forEach((step) => {
        step.patterns.forEach((pattern) => {
          if (message.includes(pattern)) {
            const status = pattern.includes("✅") ? "completed" : "in-progress";
            // Use delayed update for better UX
            updateStepWithDelay(step.id, status, message);
          }
        });
      });

      // Check for error patterns
      if (
        message.includes("❌") ||
        message.includes("Failed") ||
        message.includes("Error")
      ) {
        const failedStep = VERIFICATION_STEPS.find((step) =>
          step.patterns.some((pattern) =>
            message.includes(pattern.replace("✅", "❌"))
          )
        );

        if (failedStep) {
          updateStepWithDelay(failedStep.id, "error", message);
        }
      }
    };

    // Override console methods to capture WASM logs
    console.log = (...args: any[]) => {
      const message = args.join(" ");
      handleLogMessage(message);
      originalConsoleLog.current?.(...args);
    };

    console.info = (...args: any[]) => {
      const message = args.join(" ");
      handleLogMessage(message);
      originalConsoleInfo.current?.(...args);
    };

    return () => {
      // Restore original console methods
      if (originalConsoleLog.current) console.log = originalConsoleLog.current;
      if (originalConsoleInfo.current)
        console.info = originalConsoleInfo.current;
    };
  }, [state.isVerifying, updateStepWithDelay]);

  const resetVerification = useCallback(() => {
    setState({
      isOpen: false,
      isVerifying: false,
      allStepsCompleted: false,
      steps: VERIFICATION_STEPS.reduce(
        (acc, step) => ({
          ...acc,
          [step.id]: { status: "pending" as StepStatus },
        }),
        {}
      ),
      result: undefined,
    });
  }, []);

  const handleVerifyClick = async () => {
    resetVerification();
    setState((prev) => ({ ...prev, isOpen: true, isVerifying: true }));

    try {
      await init();

      // Fetch proof and settings
      const proofResp = await fetch(proofPath);
      const settingsResp = await fetch(settingsPath);

      if (!proofResp.ok || !settingsResp.ok) {
        throw new Error("Could not load proof or settings files");
      }

      const proofBytes = new Uint8Array(await proofResp.arrayBuffer());
      const settingsBytes = new Uint8Array(await settingsResp.arrayBuffer());

      // Run verification - this will output the console logs we're monitoring
      const result = verify(proofBytes, settingsBytes);

      setState((prev) => ({
        ...prev,
        isVerifying: false,
        result: {
          success: result.success,
          message:
            result.error_message || "Verification completed successfully!",
        },
      }));
    } catch (error) {
      console.error("Verification error:", error);

      // Mark the current step as error
      const currentStep = VERIFICATION_STEPS.find((step) => {
        const stepState = state.steps[step.id];
        return stepState.status === "in-progress";
      });

      if (currentStep) {
        updateStepWithDelay(
          currentStep.id,
          "error",
          error instanceof Error ? error.message : "Unknown error"
        );
      }

      setState((prev) => ({
        ...prev,
        isVerifying: false,
        result: {
          success: false,
          message:
            error instanceof Error ? error.message : "Unknown error occurred",
        },
      }));
    }
  };

  const getStepIcon = (status: StepStatus) => {
    switch (status) {
      case "completed":
        return <Check className="h-5 w-5 text-green-500" />;
      case "in-progress":
        return <Loader2 className="h-5 w-5 animate-spin text-gray-500" />;
      case "error":
        return <X className="h-5 w-5 text-red-500" />;
      default:
        return (
          <div className="h-5 w-5 rounded-full border-2 border-gray-300 dark:border-gray-600" />
        );
    }
  };

  const getOverallStatus = () => {
    // Show error immediately if verification failed
    if (state.result && !state.result.success) {
      return "error";
    }

    // Show success only if verification succeeded AND all step delays are completed
    if (state.result && state.result.success && state.allStepsCompleted) {
      return "completed";
    }

    // Show in-progress if we're verifying or if verification succeeded but steps are still completing
    if (
      state.isVerifying ||
      (state.result && state.result.success && !state.allStepsCompleted)
    ) {
      return "in-progress";
    }

    return "pending";
  };

  const getCircleIcon = () => {
    const status = getOverallStatus();
    switch (status) {
      case "completed":
        return <Check className="h-12 w-12 text-green-600" />;
      case "in-progress":
        return <Loader2 className="h-12 w-12 animate-spin text-gray-500" />;
      case "error":
        return <AlertCircle className="h-12 w-12 text-red-600" />;
      default:
        return (
          <div className="h-12 w-12 rounded-full border-4 border-gray-300 dark:border-gray-600" />
        );
    }
  };

  return (
    <>
      <Button
        onClick={handleVerifyClick}
        disabled={state.isVerifying}
        className={cn(
          "relative bg-black text-white hover:bg-gray-800 dark:bg-white dark:text-black dark:hover:bg-gray-200 transition-colors font-mono text-sm px-6 py-3 rounded-sm border border-gray-200 dark:border-gray-700",
          className
        )}
      >
        {state.isVerifying ? (
          <>
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            VERIFYING...
          </>
        ) : (
          <>
            {buttonText}
            <svg
              className="ml-2 h-4 w-4"
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
          </>
        )}
      </Button>

      <Dialog
        open={state.isOpen}
        onOpenChange={(open) => {
          if (!open && !state.isVerifying) {
            resetVerification();
          }
        }}
      >
        <DialogContent className="sm:max-w-[900px] font-sans bg-white dark:bg-black border-gray-200 dark:border-gray-700">
          <DialogHeader className="pb-3">
            <DialogTitle className="text-2xl font-bold text-left mb-2 text-gray-900 dark:text-gray-100">
              {title}
            </DialogTitle>
          </DialogHeader>

          {/* Two-column layout with terminal design */}
          <div className="flex flex-col md:flex-row gap-8 min-h-[400px]">
            {/* Left column - Terminal and controls (second on mobile, first on desktop) */}
            <div className="flex-1 flex flex-col space-y-4 order-2 md:order-1">
              {/* Terminal container */}
              <div className="bg-white dark:bg-gray-950 rounded-lg p-4 font-mono text-sm min-h-[300px] border border-gray-800">
                <div className="flex items-center mb-3 pb-2 border-b border-gray-700">
                  <div className="text-gray-400 text-xs">verification logs</div>
                </div>

                <div className="space-y-1">
                  {VERIFICATION_STEPS.map((step, index) => {
                    const stepState = state.steps[step.id];
                    const isActive = stepState.status === "in-progress";
                    const isCompleted = stepState.status === "completed";
                    const isError = stepState.status === "error";

                    return (
                      <div
                        key={step.id}
                        className={cn(
                          "flex items-center space-x-2 py-1 transition-all",
                          isActive && "animate-pulse",
                          stepState.status === "pending" &&
                            !state.isVerifying &&
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
                            stepState.status === "pending" && "text-gray-700",
                            stepState.status === "in-progress" &&
                              "text-gray-600",
                            stepState.status === "completed" && "text-gray-500",
                            stepState.status === "error" && "text-red-400"
                          )}
                        >
                          {step.title}
                        </span>
                      </div>
                    );
                  })}

                  {state.isVerifying && (
                    <div className="flex items-center space-x-2 py-1">
                      <Loader2 className="h-3 w-3 animate-spin text-blue-400" />
                      <span className="text-blue-400 text-xs">
                        Processing verification...
                      </span>
                    </div>
                  )}

                  {(getOverallStatus() === "completed" ||
                    (state.result && !state.result.success)) && (
                    <div className="flex items-center space-x-2 py-1 mt-2 pt-2 border-t border-gray-700">
                      <span
                        className={cn(
                          state.result?.success
                            ? "text-green-400"
                            : "text-red-400"
                        )}
                      >
                        {state.result?.success ? "✓" : "✗"}
                      </span>
                      <span
                        className={cn(
                          "text-xs",
                          state.result?.success
                            ? "text-green-400"
                            : "text-red-400"
                        )}
                      >
                        {state.result?.success
                          ? "Verification completed successfully"
                          : `Verification failed: ${state.result?.message}`}
                      </span>
                    </div>
                  )}
                </div>
              </div>

              {/* Download button */}
              {state.result && (
                <Button
                  variant="outline"
                  className="text-sm font-mono border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 self-start"
                  onClick={async () => {
                    try {
                      const zip = new JSZip();

                      // Fetch the files
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

                      // Generate and download the zip
                      const zipBlob = await zip.generateAsync({ type: "blob" });
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
                  Download proof
                </Button>
              )}

              {/* Footer information */}
              <div className="mt-auto space-y-2">
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

            {/* Vertical divider - hidden on mobile */}
            <div className="hidden md:block w-px bg-gray-200 dark:bg-gray-700 my-4 order-1 md:order-2"></div>

            {/* Right column - Status circle and description (first on mobile, last on desktop) */}
            <div className="flex-1 flex flex-col items-center justify-start pt-8 order-1 md:order-3">
              <div className="text-left space-y-3 w-full">
                <DialogDescription className="text-sm text-gray-600 dark:text-gray-300">
                  You are verifying a{" "}
                  <a
                    href="https://luminair.gizatech.xyz/"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-green-600 dark:text-green-400 hover:underline"
                  >
                    LuminAIR
                  </a>{" "}
                  Circle STARK proof entirely within your browser—no external
                  network requests are made during verification. This
                  cryptographic proof ensures the computational integrity of the
                  model inference.
                </DialogDescription>

                {/* Additional fields */}
                <div className="space-y-2 pt-2 border-t border-gray-200 dark:border-gray-700">
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
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
