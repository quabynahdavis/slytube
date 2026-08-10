<script setup lang="ts">
import { AlertDialogDescription, useForwardProps } from "reka-ui";
import type { AlertDialogDescriptionProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AlertDialogDescriptionProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AlertDialogDescription
    v-bind="forwarded"
    :class="cn('text-sm text-muted-foreground', props.class)"
  >
    <slot />
  </AlertDialogDescription>
</template>
