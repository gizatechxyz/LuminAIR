"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { ChevronRight } from "lucide-react";
import { cn } from "../lib/utils";
import { Badge } from "./ui/badge";
import init, { verify } from "@gizatech/luminair-web";
import {
  VerificationModal,
  VERIFICATION_STEPS,
  type VerificationState,
  type StepStatus,
} from "./VerificationModal";

export interface VerifyBadgeProps {
  /** Path to the proof file (required) */
  proofPath: string;
  /** Path to the settings file (required) */
  settingsPath: string;
  /** Title displayed in the modal (default: "Can't be evil.") */
  title?: string;
  /** Text displayed on the badge (default: "VERIFIED COMPUTE") */
  labelText?: string;
  /** Author name displayed in the modal (default: "Giza") */
  author?: string;
  /** Model description displayed in the modal (default: "Demo model") */
  modelDescription?: string;
  /** Author URL (default: "https://www.gizatech.xyz/") */
  authorUrl?: string;
  /** Custom className for the badge */
  className?: string;
  /** Badge variant (default: "default") */
  variant?: "default" | "secondary" | "destructive" | "outline";
}

export function VerifyBadge({
  proofPath,
  settingsPath,
  title = "Can't be evil.",
  labelText = "VERIFIED COMPUTE",
  author = "Giza",
  modelDescription = "Demo model",
  authorUrl = "https://www.gizatech.xyz/",
  className,
  variant = "default",
}: VerifyBadgeProps) {
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

    // Get logo color based on status - light green but darker than background
    const getLogoColor = () => {
      switch (status) {
        case "completed":
          return "text-green-800 dark:text-green-300";
        case "error":
          return "text-red-300 dark:text-red-300";
        case "in-progress":
          return "text-amber-300 dark:text-amber-300";
        default:
          return "text-amber-300 dark:text-amber-300";
      }
    };

    const logoColorClass = getLogoColor();

    return (
      <div className="relative mr-2">
        <svg
          className={cn("h-4 w-4", logoColorClass)}
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

  const getBadgeVariant = () => {
    const status = getOverallStatus();

    switch (status) {
      case "completed":
        return variant;
      case "error":
        return "destructive";
      case "in-progress":
        return "default";
      default:
        return "default";
    }
  };

  // Get badge colors based on status
  const getBadgeColors = () => {
    const status = getOverallStatus();

    switch (status) {
      case "completed":
        return "bg-green-100 dark:bg-green-950/50 text-green-800 dark:text-green-300 border-green-800 dark:border-green-300 hover:bg-green-200 dark:hover:bg-green-900";
      case "error":
        return "bg-red-50 dark:bg-red-950 text-red-600 dark:text-red-500 border-red-600 dark:border-red-500 hover:bg-red-100 dark:hover:bg-red-900";
      case "in-progress":
        return "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-500 border-amber-600 dark:border-amber-500 hover:bg-amber-100 dark:hover:bg-amber-900";
      default:
        return "bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-500 border-amber-600 dark:border-amber-500 hover:bg-amber-100 dark:hover:bg-amber-900";
    }
  };

  return (
    <>
      <Badge
        onClick={handleLabelClick}
        variant={getBadgeVariant()}
        className={cn(
          "cursor-pointer hover:shadow-md transition-all duration-200 px-3 py-1 text-xs font-mono rounded-md",
          "flex items-center justify-between min-w-0 w-fit max-w-xs border",
          getBadgeColors(),
          state.isVerifying && "opacity-75",
          className
        )}
      >
        <div className="flex items-center min-w-0 flex-1">
          {getGizaLogoWithStatus()}

          <div className="flex flex-col items-start min-w-0 flex-1">
            <span className="font-mono text-xs leading-tight font-medium">
              {getStatusText()}
            </span>
          </div>
        </div>

        {/* Chevron indicator */}
        <ChevronRight className="h-3 w-3 ml-2 flex-shrink-0" />
      </Badge>

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
