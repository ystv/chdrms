import { createGroup } from '#/client';
import { listGroupsOptions } from '#/client/@tanstack/react-query.gen';
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
import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import { revalidateLogic } from '@tanstack/react-form';
import { zCreateGroup } from '#/client/zod.gen';
import { useDisclosure } from '@mantine/hooks';
import { useState } from 'react';

export const Route = createFileRoute('/(authenticated)/groups/')({
  component: RouteComponent,
});

function RouteComponent() {
  const groups = useQuery({ ...listGroupsOptions() });

  const [
    createModalOpened,
    { open: openCreateModal, close: closeCreateModal },
  ] = useDisclosure(false);

  return (
    <Stack>
      <Group>
        <Title>Groups</Title>
        <Button.Group ml={'auto'}>
          <Button onClick={openCreateModal}>Create</Button>
        </Button.Group>
      </Group>
      <CreateGroupModal
        opened={createModalOpened}
        onClose={closeCreateModal}
        onCreate={groups.refetch}
      />
      <Table striped>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Group</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {groups.data?.map((group) => (
            <Table.Tr key={group.id}>
              <Table.Td>
                <Group>{group.name}</Group>
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}

function CreateGroupModal(props: {
  opened: boolean;
  onClose: () => void;
  onCreate: () => void;
}) {
  const [createMore, setCreateMore] = useState(false);

  const form = useAppForm({
    defaultValues: {
      name: '',
    },
    validationLogic: revalidateLogic(),
    validators: {
      onDynamic: zCreateGroup,
    },
    onSubmit: async ({ value }) => {
      const res = await createGroup({ body: value });

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
          children={(field) => <field.TextField label="Group Name" />}
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
