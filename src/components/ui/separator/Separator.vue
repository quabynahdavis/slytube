<script setup lang="ts">
import { Separator, useForwardProps } from "reka-ui";
import type { SeparatorProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends SeparatorProps {
  class?: string;
  label?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <Separator
    v-bind="forwarded"
    :class="
      cn(
        'shrink-0 bg-border',
        props.orientation === 'vertical' ? 'h-full w-px' : 'h-px w-full',
        props.class
      )
    "
  />
</template>
