<script setup lang="ts">
import { AlertDialogTitle, useForwardProps } from "reka-ui";
import type { AlertDialogTitleProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AlertDialogTitleProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AlertDialogTitle
    v-bind="forwarded"
    :class="cn('text-lg font-semibold', props.class)"
  >
    <slot />
  </AlertDialogTitle>
</template>
