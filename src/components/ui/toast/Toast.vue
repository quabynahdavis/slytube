<script setup lang="ts">
import {
  ToastRoot,
  useForwardPropsEmits,
} from "reka-ui";
import type { ToastRootEmits, ToastRootProps } from "reka-ui";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const toastVariants = cva(
  "group pointer-events-auto relative flex w-full items-center justify-between space-x-2 overflow-hidden rounded-md border p-4 pr-6 shadow-lg transition-all data-[swipe=cancel]:translate-x-0 data-[swipe=end]:translate-x-[var(--reka-toast-swipe-end-x)] data-[swipe=move]:translate-x-[var(--reka-toast-swipe-move-x)] data-[swipe=move]:transition-none data-[state=open]:animate-in data-[state=closed]:animate-out data-[swipe=end]:animate-out data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-top-full data-[state=open]:sm:slide-in-from-bottom-full",
  {
    variants: {
      variant: {
        default: "border bg-background text-foreground",
        destructive:
          "destructive group border-destructive bg-destructive text-destructive-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

interface Props extends ToastRootProps {
  class?: string;
  variant?: VariantProps<typeof toastVariants>["variant"];
  onOpenChange?: ((value: boolean) => void) | undefined;
}

const props = defineProps<Props>();
const emits = defineEmits<ToastRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <ToastRoot
    v-bind="forwarded"
    :class="cn(toastVariants({ variant }), props.class)"
    @open-change="onOpenChange"
  >
    <slot />
  </ToastRoot>
</template>
