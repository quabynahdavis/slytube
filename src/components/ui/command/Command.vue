<script setup lang="ts">
import { ComboboxRoot, useForwardPropsEmits } from "reka-ui";
import type { ComboboxRootEmits, ComboboxRootProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ComboboxRootProps {
  class?: string;
}

const props = defineProps<Props>();
const emits = defineEmits<ComboboxRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <ComboboxRoot
    v-bind="forwarded"
    :class="
      cn(
        'flex h-full w-full flex-col overflow-hidden rounded-md bg-popover text-popover-foreground',
        props.class
      )
    "
  >
    <slot />
  </ComboboxRoot>
</template>
