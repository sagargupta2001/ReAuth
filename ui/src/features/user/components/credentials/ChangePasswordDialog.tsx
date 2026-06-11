import { useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { useUpdateUserPassword } from '@/features/user/api/useUserCredentials'
import { ApiError } from '@/shared/api/client'
import { Checkbox } from '@/shared/ui/checkbox'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Separator } from '@/shared/ui/separator'

const changePasswordSchema = z
  .object({
    password: z.string(),
    confirm_password: z.string(),
    sign_out_all_sessions: z.boolean(),
    skip_password_checks: z.boolean(),
  })
  .superRefine((values, ctx) => {
    if (!values.password) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'New password is required',
        path: ['password'],
      })
    }

    if (!values.skip_password_checks && values.password.length < 8) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'Password must be at least 8 characters',
        path: ['password'],
      })
    }

    if (values.password.length > 100) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'Password must be no more than 100 characters',
        path: ['password'],
      })
    }

    if (values.password !== values.confirm_password) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: 'Passwords do not match',
        path: ['confirm_password'],
      })
    }
  })

type ChangePasswordFormValues = z.infer<typeof changePasswordSchema>

interface ChangePasswordDialogProps {
  userId: string
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function ChangePasswordDialog({ userId, open, onOpenChange }: ChangePasswordDialogProps) {
  const mutation = useUpdateUserPassword(userId)
  const [serverError, setServerError] = useState<string | null>(null)

  const form = useForm<ChangePasswordFormValues>({
    resolver: zodResolver(changePasswordSchema),
    defaultValues: {
      password: '',
      confirm_password: '',
      sign_out_all_sessions: false,
      skip_password_checks: false,
    },
  })

  const handleOpenChange = (nextOpen: boolean) => {
    onOpenChange(nextOpen)
    if (!nextOpen) {
      form.reset()
      setServerError(null)
    }
  }

  const onSubmit = (values: ChangePasswordFormValues) => {
    setServerError(null)
    mutation.mutate(
      {
        password: values.password,
        sign_out_all_sessions: values.sign_out_all_sessions,
        skip_password_checks: values.skip_password_checks,
      },
      {
        onSuccess: () => handleOpenChange(false),
        onError: (error) => {
          if (error instanceof ApiError) {
            form.setError('password', { type: 'server', message: error.message })
            setServerError(error.message)
          }
        },
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Change password</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
            <FormInput
              control={form.control}
              name="password"
              label="New password"
              type="password"
              placeholder="Enter new password"
            />
            <FormInput
              control={form.control}
              name="confirm_password"
              label="Confirm password"
              type="password"
              placeholder="Re-enter new password"
            />

            <div className="mt-2 flex items-start space-x-3">
              <Checkbox
                id="sign_out_all_sessions"
                checked={form.watch('sign_out_all_sessions')}
                onCheckedChange={(checked) =>
                  form.setValue('sign_out_all_sessions', checked as boolean)
                }
              />
              <div className="-mt-0.5 grid gap-1.5 leading-none">
                <label
                  htmlFor="sign_out_all_sessions"
                  className="cursor-pointer text-[14px] leading-none font-medium"
                >
                  Sign out of all sessions
                </label>
                <p className="text-muted-foreground mt-1 text-[13px]">
                  Revoke all active refresh sessions for this user after updating the password.
                </p>
              </div>
            </div>

            <div className="flex items-start space-x-3">
              <Checkbox
                id="skip_password_checks"
                checked={form.watch('skip_password_checks')}
                onCheckedChange={(checked) =>
                  form.setValue('skip_password_checks', checked as boolean)
                }
              />
              <div className="-mt-0.5 grid gap-1.5 leading-none">
                <label
                  htmlFor="skip_password_checks"
                  className="cursor-pointer text-[14px] leading-none font-medium"
                >
                  Skip password checks
                </label>
                <p className="text-muted-foreground mt-1 text-[13px]">
                  Allow this password update to bypass the normal minimum length check.
                </p>
              </div>
            </div>

            {serverError ? <p className="text-destructive text-sm">{serverError}</p> : null}
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={form.handleSubmit(onSubmit)} disabled={mutation.isPending}>
              {mutation.isPending ? 'Changing...' : 'Change password'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
