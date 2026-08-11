import { createLocation } from '#/client';
import type { Location } from '#/client';
import { listLocationsOptions } from '#/client/@tanstack/react-query.gen';
import { zCreateLocationBody } from '#/client/zod.gen';
import { useAppForm } from '#/components/form';
import {
  Button,
  Checkbox,
  Group,
  Modal,
  Stack,
  Table,
  Title,
} from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import { revalidateLogic } from '@tanstack/react-form';
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import { useState } from 'react';

export const Route = createFileRoute('/(authenticated)/locations/')({
  component: RouteComponent,
});

function RouteComponent() {
  const locations = useQuery({ ...listLocationsOptions() });

  const [
    createModalOpened,
    { open: openCreateModal, close: closeCreateModal },
  ] = useDisclosure(false);

  return (
    <Stack>
      <Group>
        <Title>Locations</Title>
        <Button.Group ml={'auto'}>
          <Button onClick={openCreateModal}>Create</Button>
        </Button.Group>
      </Group>
      <CreateLocationModal
        opened={createModalOpened}
        onClose={closeCreateModal}
        onCreate={locations.refetch}
      />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {locations.data?.map((location) => (
            <Table.Tr key={location.id}>
              <Table.Td>
                <Group>{location.name}</Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}

function CreateLocationModal(props: {
  opened: boolean;
  onClose: () => void;
  onCreate: () => void;
}) {
  const [createMore, setCreateMore] = useState(false);

  const defaultLocation: Location = {
    name: '',
  };

  const form = useAppForm({
    defaultValues: defaultLocation,
    validationLogic: revalidateLogic(),
    validators: {
      onDynamic: zCreateLocationBody,
    },
    onSubmit: async ({ value }) => {
      const res = await createLocation({ body: value });

      if (res.data) {
        props.onCreate();
        if (!createMore) {
          props.onClose();
        }
        form.reset();
      }
    },
  });

  return (
    <Modal opened={props.opened} onClose={props.onClose}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          form.handleSubmit();
        }}
      >
        <form.AppField
          name="name"
          children={(field) => <field.TextField label="Name" />}
        />

        <form.AppField
          name="description"
          children={(field) => <field.TextField label="Description" />}
        />

        <form.AppForm>
          <form.SubscribeButton children="Submit" />
        </form.AppForm>
      </form>
      <Checkbox
        mt={6}
        checked={createMore}
        onChange={(event) => setCreateMore(event.currentTarget.checked)}
        label="Create more?"
      />
    </Modal>
  );
}
