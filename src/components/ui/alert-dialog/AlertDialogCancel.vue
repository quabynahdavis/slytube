<script setup lang="ts">
import { AlertDialogCancel, useForwardProps } from "reka-ui";
import type { AlertDialogCancelProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AlertDialogCancelProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AlertDialogCancel
    v-bind="forwarded"
    :class="cn('mt-2 inline-flex h-9 items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 sm:mt-0', props.class)"
  >
    <slot />
  </AlertDialogCancel>
</template>
