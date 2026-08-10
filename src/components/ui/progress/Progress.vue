<script setup lang="ts">
import {
  ProgressIndicator,
  ProgressRoot,
  useForwardPropsEmits,
} from "reka-ui";
import type { ProgressRootEmits, ProgressRootProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ProgressRootProps {
  class?: string;
}

const props = defineProps<Props>();
const emits = defineEmits<ProgressRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <ProgressRoot
    v-bind="forwarded"
    :class="
      cn(
        'relative h-2 w-full overflow-hidden rounded-full bg-primary/20',
        props.class
      )
    "
  >
    <ProgressIndicator
      class="h-full w-full flex-1 bg-primary transition-all"
      :style="`transform: translateX(-${100 - ((forwarded as any).value || 0)}%);`"
    />
  </ProgressRoot>
</template>
