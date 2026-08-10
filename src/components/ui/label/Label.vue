<script setup lang="ts">
import { Label, useForwardProps } from "reka-ui";
import type { LabelProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends LabelProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <Label
    v-bind="forwarded"
    :class="
      cn(
        'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
        props.class
      )
    "
  >
    <slot />
  </Label>
</template>
