<script setup lang="ts">
import { ComboboxItem, useForwardPropsEmits } from "reka-ui";
import type { ComboboxItemEmits, ComboboxItemProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ComboboxItemProps {
  class?: string;
}

const props = defineProps<Props>();
const emits = defineEmits<ComboboxItemEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <ComboboxItem
    v-bind="forwarded"
    :class="
      cn(
        'relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.class
      )
    "
  >
    <slot />
  </ComboboxItem>
</template>
