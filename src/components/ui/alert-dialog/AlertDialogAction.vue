<script setup lang="ts">
import { AlertDialogAction, useForwardProps } from "reka-ui";
import type { AlertDialogActionProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AlertDialogActionProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AlertDialogAction
    v-bind="forwarded"
    :class="cn('inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50', props.class)"
  >
    <slot />
  </AlertDialogAction>
</template>
