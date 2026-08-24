import { createAssetType } from '#/client';
import type { AssetType } from '#/client';
import { listAssetTypesOptions } from '#/client/@tanstack/react-query.gen';
import { zCreateAssetTypeBody } from '#/client/zod.gen';
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

export const Route = createFileRoute('/(authenticated)/asset-types/')({
  component: RouteComponent,
});

function RouteComponent() {
  const assetTypes = useQuery({ ...listAssetTypesOptions() });

  const [
    createModalOpened,
    { open: openCreateModal, close: closeCreateModal },
  ] = useDisclosure(false);

  return (
    <Stack>
      <Group>
        <Title>Asset Types</Title>
        <Button.Group ml={'auto'}>
          <Button onClick={openCreateModal}>Create</Button>
        </Button.Group>
      </Group>
      <CreateAssetTypeModal
        opened={createModalOpened}
        onClose={closeCreateModal}
        onCreate={assetTypes.refetch}
      />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Name</Table.Th>
            <Table.Th>Value</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {assetTypes.data?.map((assetType) => (
            <Table.Tr key={assetType.id}>
              <Table.Td>{assetType.name}</Table.Td>
              <Table.Td>{assetType.value}</Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}

function CreateAssetTypeModal(props: {
  opened: boolean;
  onClose: () => void;
  onCreate: () => void;
}) {
  const [createMore, setCreateMore] = useState(false);

  const defaultAssetType: AssetType = {
    name: '',
    manufacturer: '',
    product_url: null,
    value: null,
  };

  const form = useAppForm({
    defaultValues: defaultAssetType,
    validationLogic: revalidateLogic(),
    validators: {
      onDynamic: zCreateAssetTypeBody,
    },
    onSubmit: async ({ value }) => {
      const res = await createAssetType({ body: value });

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
          children={(field) => (
            <field.TextField label="Asset Type Name" required />
          )}
        />

        <form.AppField
          name="value"
          children={(field) => (
            <field.NumberAsStringField
              label="Value"
              decimalScale={2}
              min={0}
              fixedDecimalScale
              leftSection={'£'}
              required
            />
          )}
        />

        <form.AppField
          name="manufacturer"
          children={(field) => (
            <field.ManufacturerField label="Manufacturer" required />
          )}
        />

        <form.AppField
          name="product_url"
          children={(field) => <field.TextField label="Product URL" />}
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
