import { useSelector } from '@tanstack/react-form';
import { useFieldContext } from '../context.tsx';
import { TextInput } from '@mantine/core';

export default function TextField({ label }: { label: string }) {
  const field = useFieldContext<string>();

  const errors = useSelector(field.store, (state) => state.meta.errors);

  return (
    <TextInput
      label={label}
      value={field.state.value}
      onChange={(e) => field.handleChange(e.target.value)}
      onBlur={field.handleBlur}
      error={errors[0]?.message}
    />
  );
}
