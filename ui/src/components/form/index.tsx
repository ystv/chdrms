import { createFormHook, createFormHookContexts } from '@tanstack/react-form';
import { lazy } from 'react';
import { useFormContext } from './context';
import { Button, Group } from '@mantine/core';
import type { ButtonProps } from '@mantine/core';

const TextField = lazy(() => import('./fields/text'));

const { fieldContext, formContext } = createFormHookContexts();

function SubscribeButton(props: ButtonProps) {
  const form = useFormContext();
  return (
    <form.Subscribe selector={(state) => state.isSubmitting}>
      {(isSubmitting) => (
        <Group>
          <Button
            mt={4}
            ml={'auto'}
            {...props}
            disabled={isSubmitting}
            onClick={form.handleSubmit}
          />
        </Group>
      )}
    </form.Subscribe>
  );
}

export const { useAppForm } = createFormHook({
  fieldComponents: { TextField },
  formComponents: { SubscribeButton },
  fieldContext,
  formContext,
});
