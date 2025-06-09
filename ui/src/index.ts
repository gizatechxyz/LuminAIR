// Export the main component
export { VerifyButton } from "./components/VerifyButton";
export type { VerifyButtonProps } from "./components/VerifyButton";

// Export the VerifyBadge component
export { VerifyBadge } from "./components/VerifyBadge";
export type { VerifyBadgeProps } from "./components/VerifyBadge";

// Export the shared VerificationModal component
export { VerificationModal, VERIFICATION_STEPS } from "./components/VerificationModal";
export type { VerificationState, StepState, StepStatus } from "./components/VerificationModal";

// Export the graph visualizer component
export { GraphVisualizer } from "./components/GraphVisualizer";

// Export UI components in case users want to use them separately
export { Button } from "./components/ui/button";
export { Badge } from "./components/ui/badge";
export { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "./components/ui/dialog";

// Export utilities
export { cn } from "./lib/utils";
export { getSharedButtonStyles, baseButtonStyles } from "./lib/shared-styles"; 