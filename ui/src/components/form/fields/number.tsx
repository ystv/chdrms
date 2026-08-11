import { useSelector } from '@tanstack/react-form';
import { useFieldContext } from '../context.tsx';
import { NumberInput } from '@mantine/core';
import type { NumberInputProps } from '@mantine/core';

export default function NumberField(props: NumberInputProps) {
  const field = useFieldContext<string | number>();

  const errors = useSelector(field.store, (state) => state.meta.errors);

  return (
    <NumberInput
      {...props}
      value={field.state.value}
      onChange={(e) => field.handleChange(e.valueOf())}
      onBlur={field.handleBlur}
      error={errors[0]?.message}
    />
  );
}
