<script setup lang="ts">
import {
  ScrollAreaScrollbar,
  ScrollAreaThumb,
  useForwardProps,
} from "reka-ui";
import type { ScrollAreaScrollbarProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ScrollAreaScrollbarProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ScrollAreaScrollbar
    v-bind="forwarded"
    :class="
      cn(
        'flex touch-none select-none transition-colors',
        props.orientation === 'vertical'
          ? 'h-full w-2.5 border-l border-l-transparent p-[1px]'
          : 'h-2.5 flex-col border-t border-t-transparent p-[1px]',
        props.class
      )
    "
  >
    <ScrollAreaThumb class="relative flex-1 rounded-full bg-border" />
  </ScrollAreaScrollbar>
</template>
