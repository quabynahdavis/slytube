<script setup lang="ts">
import {
  CheckboxIndicator,
  CheckboxRoot,
  useForwardPropsEmits,
} from "reka-ui";
import type { CheckboxRootEmits, CheckboxRootProps } from "reka-ui";
import { cn } from "@/lib/utils";
import { HugeiconsIcon } from "@hugeicons/vue";
import { Tick01Icon } from "@hugeicons/core-free-icons";

interface Props extends CheckboxRootProps {
  class?: string;
}

const props = defineProps<Props>();
const emits = defineEmits<CheckboxRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <CheckboxRoot
    v-bind="forwarded"
    :class="
      cn(
        'peer h-4 w-4 shrink-0 rounded-sm border border-primary shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground',
        props.class
      )
    "
  >
    <CheckboxIndicator
      class="flex items-center justify-center text-current"
    >
      <HugeiconsIcon :icon="Tick01Icon" class="h-4 w-4" />
    </CheckboxIndicator>
  </CheckboxRoot>
</template>
