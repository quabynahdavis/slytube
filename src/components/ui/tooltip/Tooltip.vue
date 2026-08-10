<script setup lang="ts">
import { TooltipRoot, useForwardPropsEmits } from "reka-ui";
import type { TooltipRootEmits, TooltipRootProps } from "reka-ui";

interface Props extends TooltipRootProps {}

const props = defineProps<Props>();
const emits = defineEmits<TooltipRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <TooltipRoot v-bind="forwarded">
    <slot />
  </TooltipRoot>
</template>
