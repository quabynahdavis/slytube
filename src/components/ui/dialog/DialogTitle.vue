<script setup lang="ts">
import { DialogTitle, useForwardProps } from "reka-ui";
import type { DialogTitleProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends DialogTitleProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <DialogTitle
    v-bind="forwarded"
    :class="cn('text-lg font-semibold leading-none tracking-tight', props.class)"
  >
    <slot />
  </DialogTitle>
</template>
