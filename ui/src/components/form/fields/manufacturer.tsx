import { useSelector } from '@tanstack/react-form';
import { useFieldContext } from '../context.tsx';
import { Select } from '@mantine/core';
import type { SelectProps } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { listManufacturersOptions } from '#/client/@tanstack/react-query.gen.ts';

export default function ManufacturerField(props: SelectProps) {
  const field = useFieldContext<string | null>();

  const manufacturers = useQuery({ ...listManufacturersOptions() });

  const errors = useSelector(field.store, (state) => state.meta.errors);

  return (
    <Select
      {...props}
      data={manufacturers.data?.map((m) => ({
        value: m.id,
        label: m.name,
      }))}
      searchable
      value={field.state.value}
      onChange={(e) => field.handleChange(e)}
      onBlur={field.handleBlur}
      error={errors[0]?.message}
    />
  );
}
