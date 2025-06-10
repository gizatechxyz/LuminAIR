"use client";

import React, { useState, useCallback, useEffect, useRef } from "react";
import { Button } from "./ui/button";
import { Check, Loader2, X, AlertCircle } from "lucide-react";
import { cn } from "../lib/utils";
import { getSharedButtonStyles } from "../lib/shared-styles";
import init, { verify } from "@gizatech/luminair-web";
import {
  VerificationModal,
  VERIFICATION_STEPS,
  type VerificationState,
  type StepStatus,
} from "./VerificationModal";

export interface VerifyButtonProps {
  /** Path to the proof file (required) */
  proofPath: string;
  /** Path to the settings file (required) */
  settingsPath: string;
  /** Path to the graph visualization file (required) */
  graphPath: string;
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
  graphPath,
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

  return (
    <>
      <Button
        onClick={handleVerifyClick}
        disabled={state.isVerifying}
        className={cn(getSharedButtonStyles(), className)}
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

      <VerificationModal
        isOpen={state.isOpen}
        onOpenChange={(open) => {
          if (!open && !state.isVerifying) {
            resetVerification();
          }
        }}
        verificationState={state}
        proofPath={proofPath}
        settingsPath={settingsPath}
        graphPath={graphPath}
        title={title}
        author={author}
        modelDescription={modelDescription}
        authorUrl={authorUrl}
      />
    </>
  );
}
