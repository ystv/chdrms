import { useSelector } from '@tanstack/react-form';
import { useFieldContext } from '../context.tsx';
import { Select } from '@mantine/core';
import type { SelectProps } from '@mantine/core';
import { useQuery } from '@tanstack/react-query';
import { listLocationsOptions } from '#/client/@tanstack/react-query.gen.ts';

export default function LocationField(props: SelectProps) {
  const field = useFieldContext<string | null>();

  const locations = useQuery({ ...listLocationsOptions() });

  const errors = useSelector(field.store, (state) => state.meta.errors);

  return (
    <Select
      {...props}
      data={locations.data?.map((l) => ({
        value: l.id,
        label: l.name,
      }))}
      searchable
      value={field.state.value}
      onChange={(e) => field.handleChange(e)}
      onBlur={field.handleBlur}
      error={errors[0]?.message}
    />
  );
}
