<script setup lang="ts">
import {
  RadioGroupIndicator,
  RadioGroupItem,
  useForwardProps,
} from "reka-ui";
import type { RadioGroupItemProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends RadioGroupItemProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <RadioGroupItem
    v-bind="forwarded"
    :class="
      cn(
        'aspect-square h-4 w-4 rounded-full border border-primary text-primary shadow focus:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50',
        props.class
      )
    "
  >
    <RadioGroupIndicator class="flex items-center justify-center">
      <div class="h-2 w-2 fill-current text-current rounded-full bg-current" />
    </RadioGroupIndicator>
  </RadioGroupItem>
</template>
