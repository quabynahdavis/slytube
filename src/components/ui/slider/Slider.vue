<script setup lang="ts">
import {
  SliderRange,
  SliderRoot,
  SliderThumb,
  SliderTrack,
  useForwardPropsEmits,
} from "reka-ui";
import type { SliderRootEmits, SliderRootProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends SliderRootProps {
  class?: string;
}

const props = defineProps<Props>();
const emits = defineEmits<SliderRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <SliderRoot
    v-bind="forwarded"
    :class="
      cn(
        'relative flex w-full touch-none select-none items-center',
        props.class
      )
    "
  >
    <SliderTrack
      class="relative h-1.5 w-full grow overflow-hidden rounded-full bg-primary/20"
    >
      <SliderRange class="absolute h-full bg-primary" />
    </SliderTrack>
    <SliderThumb
      class="block h-4 w-4 rounded-full border border-primary/50 bg-background shadow transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50"
    />
  </SliderRoot>
</template>
