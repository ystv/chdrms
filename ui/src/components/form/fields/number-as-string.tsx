import { useSelector } from '@tanstack/react-form';
import { useFieldContext } from '../context.tsx';
import { NumberInput } from '@mantine/core';
import type { NumberInputProps } from '@mantine/core';
import { useState } from 'react';

export default function NumberAsStringField(props: NumberInputProps) {
  const field = useFieldContext<string>();

  const errors = useSelector(field.store, (state) => state.meta.errors);

  const [compState, setCompState] = useState<number | string>(
    field.state.value,
  );

  return (
    <NumberInput
      {...props}
      value={compState}
      onChange={(e) => setCompState(e.valueOf())}
      onValueChange={(e) => field.handleChange(e.value)}
      onBlur={field.handleBlur}
      error={errors[0]?.message}
    />
  );
}
