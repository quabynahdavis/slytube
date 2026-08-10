<script setup lang="ts">
import {
  ScrollAreaCorner,
  ScrollAreaRoot,
  ScrollAreaViewport,
  useForwardProps,
} from "reka-ui";
import type { ScrollAreaRootProps } from "reka-ui";
import { cn } from "@/lib/utils";
import ScrollBar from "./ScrollBar.vue";

interface Props extends ScrollAreaRootProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ScrollAreaRoot
    v-bind="forwarded"
    :class="cn('relative overflow-hidden', props.class)"
  >
    <ScrollAreaViewport class="h-full w-full rounded-[inherit]">
      <slot />
    </ScrollAreaViewport>
    <ScrollBar />
    <ScrollAreaCorner />
  </ScrollAreaRoot>
</template>
