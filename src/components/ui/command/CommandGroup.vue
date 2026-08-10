<script setup lang="ts">
import { ComboboxGroup, useForwardProps } from "reka-ui";
import type { ComboboxGroupProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ComboboxGroupProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ComboboxGroup
    v-bind="forwarded"
    :class="
      cn(
        'overflow-hidden p-1 text-foreground [&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-muted-foreground',
        props.class
      )
    "
  >
    <slot />
  </ComboboxGroup>
</template>
