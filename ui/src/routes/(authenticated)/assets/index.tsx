import { createAsset } from '#/client';
import type { CreateAssetRequest } from '#/client';
import { listAssetsOptions } from '#/client/@tanstack/react-query.gen';
import { zCreateAssetBody } from '#/client/zod.gen';
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

export const Route = createFileRoute('/(authenticated)/assets/')({
  component: RouteComponent,
});

function RouteComponent() {
  const assets = useQuery({ ...listAssetsOptions() });

  const [
    createModalOpened,
    { open: openCreateModal, close: closeCreateModal },
  ] = useDisclosure(false);

  return (
    <Stack>
      <Group>
        <Title>Assets</Title>
        <Button.Group ml={'auto'}>
          <Button onClick={openCreateModal}>Create</Button>
        </Button.Group>
      </Group>
      <CreateAssetModal
        opened={createModalOpened}
        onClose={closeCreateModal}
        onCreate={assets.refetch}
      />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Tag</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {assets.data?.map((asset) => (
            <Table.Tr key={asset.id}>
              <Table.Td>
                <Group>{asset.tag}</Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}

function CreateAssetModal(props: {
  opened: boolean;
  onClose: () => void;
  onCreate: () => void;
}) {
  const [createMore, setCreateMore] = useState(false);

  const defaultAssetType: CreateAssetRequest = {
    tag: '',
    type: '',
    locations: {
      current: '',
      home: '',
    },
  };

  const form = useAppForm({
    defaultValues: defaultAssetType,
    validationLogic: revalidateLogic(),
    validators: {
      onDynamic: zCreateAssetBody,
    },
    onSubmit: async ({ value }) => {
      const res = await createAsset({ body: value });

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
          name="tag"
          children={(field) => <field.TextField label="Asset Tag" required />}
        />

        <form.AppField
          name="type"
          children={(field) => <field.AssetTypeField label="Type" required />}
        />

        <form.AppField
          name="locations.home"
          children={(field) => (
            <field.LocationField label="Home Location" required />
          )}
        />

        <form.AppField
          name="locations.current"
          children={(field) => (
            <field.LocationField label="Current Location" required />
          )}
        />

        <form.AppField
          name="alias"
          children={(field) => <field.TextField label="Alias" />}
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
