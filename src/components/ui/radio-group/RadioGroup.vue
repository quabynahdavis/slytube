<script setup lang="ts">
import { RadioGroupRoot, useForwardPropsEmits } from "reka-ui";
import type { RadioGroupRootEmits, RadioGroupRootProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends RadioGroupRootProps {
  class?: string;
}

const props = defineProps<Props>();
const emits = defineEmits<RadioGroupRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <RadioGroupRoot
    v-bind="forwarded"
    :class="cn('grid gap-2', props.class)"
  >
    <slot />
  </RadioGroupRoot>
</template>
