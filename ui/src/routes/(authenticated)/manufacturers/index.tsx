import { createManufacturer } from '#/client';
import type { Manufacturer } from '#/client';
import { listManufacturersOptions } from '#/client/@tanstack/react-query.gen';
import { zCreateManufacturerBody } from '#/client/zod.gen';
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

export const Route = createFileRoute('/(authenticated)/manufacturers/')({
  component: RouteComponent,
});

function RouteComponent() {
  const manufacturers = useQuery({ ...listManufacturersOptions() });

  const [
    createModalOpened,
    { open: openCreateModal, close: closeCreateModal },
  ] = useDisclosure(false);

  return (
    <Stack>
      <Group>
        <Title>Manufacturers</Title>
        <Button.Group ml={'auto'}>
          <Button onClick={openCreateModal}>Create</Button>
        </Button.Group>
      </Group>
      <CreateManufacturerModal
        opened={createModalOpened}
        onClose={closeCreateModal}
        onCreate={manufacturers.refetch}
      />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {manufacturers.data?.map((manufacturer) => (
            <Table.Tr key={manufacturer.id}>
              <Table.Td>
                <Group>{manufacturer.name}</Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}

function CreateManufacturerModal(props: {
  opened: boolean;
  onClose: () => void;
  onCreate: () => void;
}) {
  const [createMore, setCreateMore] = useState(false);

  const defaultManufacturer: Manufacturer = {
    name: '',
  };

  const form = useAppForm({
    defaultValues: defaultManufacturer,
    validationLogic: revalidateLogic(),
    validators: {
      onDynamic: zCreateManufacturerBody,
    },
    onSubmit: async ({ value }) => {
      const res = await createManufacturer({ body: value });

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

        <form.AppField
          name="website"
          children={(field) => <field.TextField label="Website" />}
        />

        <form.AppField
          name="email"
          children={(field) => <field.TextField label="Email" />}
        />

        <form.AppField
          name="phone"
          children={(field) => <field.TextField label="Phone" />}
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
