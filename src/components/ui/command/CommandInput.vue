<script setup lang="ts">
import { ComboboxInput, useForwardProps } from "reka-ui";
import type { ComboboxInputProps } from "reka-ui";
import { cn } from "@/lib/utils";
import { HugeiconsIcon } from "@hugeicons/vue";
import { Search01Icon } from "@hugeicons/core-free-icons";

interface Props extends ComboboxInputProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <div class="flex items-center border-b px-3" cmdk-input-wrapper="">
    <HugeiconsIcon
      :icon="Search01Icon"
      class="mr-2 h-4 w-4 shrink-0 opacity-50"
    />
    <ComboboxInput
      v-bind="forwarded"
      :class="
        cn(
          'flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50',
          props.class
        )
      "
    />
  </div>
</template>
