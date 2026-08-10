<script setup lang="ts">
import {
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  useForwardProps,
} from "reka-ui";
import type { SelectItemProps } from "reka-ui";
import { cn } from "@/lib/utils";
import { HugeiconsIcon } from "@hugeicons/vue";
import { Tick01Icon } from "@hugeicons/core-free-icons";

interface Props extends SelectItemProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <SelectItem
    v-bind="forwarded"
    :class="
      cn(
        'relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        props.class
      )
    "
  >
    <span class="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
      <SelectItemIndicator>
        <HugeiconsIcon :icon="Tick01Icon" class="h-4 w-4" />
      </SelectItemIndicator>
    </span>
    <SelectItemText>
      <slot />
    </SelectItemText>
  </SelectItem>
</template>
