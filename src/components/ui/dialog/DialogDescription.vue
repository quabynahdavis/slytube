<script setup lang="ts">
import { DialogDescription, useForwardProps } from "reka-ui";
import type { DialogDescriptionProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends DialogDescriptionProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <DialogDescription
    v-bind="forwarded"
    :class="cn('text-sm text-muted-foreground', props.class)"
  >
    <slot />
  </DialogDescription>
</template>
