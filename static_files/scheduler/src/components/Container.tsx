import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils"; 

// Define variants for the container using `cva`
const containerVariants = cva(
  "w-full mx-auto", // Base styles applied to all containers
  {
    variants: {
      maxWidth: {
        default: "max-w-screen-lg", // Default max-width
        sm: "max-w-screen-sm",
        md: "max-w-screen-md",
        lg: "max-w-screen-lg",
        xl: "max-w-screen-xl",
        full: "max-w-full",
      },
      padding: {
        none: "p-0",
        sm: "p-4",
        md: "p-8",
        lg: "p-12",
      },
    },
    defaultVariants: {
      maxWidth: "default",
      padding: "md",
    },
  }
);

// Define the types for the container component
export interface ContainerProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof containerVariants> {
  asChild?: boolean; // Allows container to be rendered as any child element
}

// Create the Container component
const Container = React.forwardRef<HTMLDivElement, ContainerProps>(
  ({ className, maxWidth, padding, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "div"; // Allows for rendering as a custom element
    return (
      <Comp
        ref={ref}
        className={cn(containerVariants({ maxWidth, padding }), className)}
        {...props}
      />
    );
  }
);

Container.displayName = "Container";

export { Container, containerVariants };

