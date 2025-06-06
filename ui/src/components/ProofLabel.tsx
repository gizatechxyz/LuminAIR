"use client";

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Check, Loader2, X, ChevronRight } from "lucide-react";
import { cn } from "../lib/utils";
import { getSharedButtonStyles } from "../lib/shared-styles";
import init, { verify } from "@gizatech/luminair-web";
import {
  VerificationModal,
  VERIFICATION_STEPS,
  type VerificationState,
  type StepStatus,
} from "./VerificationModal";

export interface ProofLabelProps {
  /** Path to the proof file (required) */
  proofPath: string;
  /** Path to the settings file (required) */
  settingsPath: string;
  /** Title displayed in the modal (default: "Can't be evil.") */
  title?: string;
  /** Text displayed on the label (default: "PROOF VERIFIED") */
  labelText?: string;
  /** Subtitle text on the label (default: "Computational Integrity") */
  subtitleText?: string;
  /** Author name displayed in the modal (default: "Giza") */
  author?: string;
  /** Model description displayed in the modal (default: "Demo model") */
  modelDescription?: string;
  /** Author URL (default: "https://www.gizatech.xyz/") */
  authorUrl?: string;
  /** Custom className for the label */
  className?: string;
}

export function ProofLabel({
  proofPath,
  settingsPath,
  title = "Can't be evil.",
  labelText = "VERIFIED COMPUTATION",
  author = "Giza",
  modelDescription = "Demo model",
  authorUrl = "https://www.gizatech.xyz/",
  className,
}: ProofLabelProps) {
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
  const verificationStarted = useRef(false);

  const updateStepStatus = useCallback(
    (stepId: string, status: StepStatus, message?: string) => {
      setState((prev) => {
        const newSteps = {
          ...prev.steps,
          [stepId]: { status, message },
        };

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

      const stepIndex = VERIFICATION_STEPS.findIndex(
        (step) => step.id === stepId
      );
      const baseDelay = 200;
      const incrementDelay = 50;
      const minDelay = baseDelay + stepIndex * incrementDelay;

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

    stepTimestamps.current = {};
    originalConsoleLog.current = console.log;
    originalConsoleInfo.current = console.info;

    const handleLogMessage = (message: string) => {
      VERIFICATION_STEPS.forEach((step) => {
        step.patterns.forEach((pattern) => {
          if (message.includes(pattern)) {
            const status = pattern.includes("✅") ? "completed" : "in-progress";
            updateStepWithDelay(step.id, status, message);
          }
        });
      });

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
      if (originalConsoleLog.current) console.log = originalConsoleLog.current;
      if (originalConsoleInfo.current)
        console.info = originalConsoleInfo.current;
    };
  }, [state.isVerifying, updateStepWithDelay]);

  // Auto-start verification on component mount
  useEffect(() => {
    if (!verificationStarted.current) {
      verificationStarted.current = true;
      startVerification();
    }
  }, []);

  const startVerification = async () => {
    setState((prev) => ({ ...prev, isVerifying: true }));

    try {
      await init();

      const proofResp = await fetch(proofPath);
      const settingsResp = await fetch(settingsPath);

      if (!proofResp.ok || !settingsResp.ok) {
        throw new Error("Could not load proof or settings files");
      }

      const proofBytes = new Uint8Array(await proofResp.arrayBuffer());
      const settingsBytes = new Uint8Array(await settingsResp.arrayBuffer());

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

  const handleLabelClick = () => {
    setState((prev) => ({ ...prev, isOpen: true }));
  };

  const getOverallStatus = () => {
    if (state.result && !state.result.success) {
      return "error";
    }

    if (state.result && state.result.success && state.allStepsCompleted) {
      return "completed";
    }

    if (
      state.isVerifying ||
      (state.result && state.result.success && !state.allStepsCompleted)
    ) {
      return "in-progress";
    }

    return "pending";
  };

  const getGizaLogoWithStatus = () => {
    const status = getOverallStatus();

    return (
      <div className="relative mr-3">
        {/* Filled Giza Logo */}
        <svg
          className="h-5 w-5"
          viewBox="0 0 18 20"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M0 14.9659L8.65331 0L17.3132 14.9659L8.65331 20L0 14.9659Z"
            fill="currentColor"
          />
        </svg>

        {/* Status Indicator positioned slightly lower in the center of the logo */}
        <div className="absolute inset-0 flex items-center justify-center translate-y-0.5">
          {status === "completed" && (
            <Check className="h-2.5 w-2.5 text-black dark:text-white" />
          )}
          {status === "in-progress" && (
            <Loader2 className="h-2.5 w-2.5 animate-spin text-black dark:text-white" />
          )}
          {status === "error" && (
            <X className="h-2.5 w-2.5 text-black dark:text-white" />
          )}
          
        </div>
      </div>
    );
  };

  const getStatusText = () => {
    const status = getOverallStatus();

    switch (status) {
      case "completed":
        return labelText;
      case "in-progress":
        return "VERIFYING...";
      case "error":
        return "VERIFICATION FAILED";
      default:
        return "VERIFYING...";
    }
  };

  return (
    <>
      <div
        onClick={handleLabelClick}
        className={cn(
          "relative bg-black text-white hover:bg-gray-800 dark:bg-white dark:text-black dark:hover:bg-gray-200 transition-colors font-mono text-sm px-4 py-2 rounded-full border border-gray-200 dark:border-gray-700",
          "cursor-pointer flex items-center justify-between min-w-0 w-fit max-w-xs shadow-sm hover:shadow-md",
          state.isVerifying && "opacity-75",
          className
        )}
      >
        <div className="flex items-center min-w-0 flex-1">
          {getGizaLogoWithStatus()}

          <div className="flex flex-col items-start min-w-0 flex-1">
            <span className="font-mono text-sm leading-tight font-medium">
              {getStatusText()}
            </span>
          </div>
        </div>

        {/* Chevron indicator */}
        <ChevronRight className="h-4 w-4 ml-2 flex-shrink-0" />
      </div>

      <VerificationModal
        isOpen={state.isOpen}
        onOpenChange={(open) => {
          if (!open && !state.isVerifying) {
            setState((prev) => ({ ...prev, isOpen: false }));
          }
        }}
        verificationState={state}
        proofPath={proofPath}
        settingsPath={settingsPath}
        title={title}
        author={author}
        modelDescription={modelDescription}
        authorUrl={authorUrl}
      />
    </>
  );
}
