// @decision Webhook is payment source of truth: Payment status must be verified asynchronously via provider webhook.
// @constraint Idempotency: All payment charge calls must supply a deterministic idempotency key.
// @warning Avoid retrying non-idempotent charges directly to prevent double billing.
// @rule Discount calculation: VIP customers receive dynamic promotional discounts.

export async function getOrder(orderId: string) {
    return { id: orderId, total: 1000 };
}

export function calculateDiscount(order: any) {
    return 100;
}

export async function charge(amount: number) {
    return { success: true, transactionId: "txn_123" };
}

export async function saveOrder(order: any, payment: any) {
    return true;
}

export async function rollback(order: any) {
    return true;
}

export async function checkout(orderId: string) {
    const order = await getOrder(orderId);
    if (!order) throw new Error("OrderNotFound");
    const discount = calculateDiscount(order);
    const amount = order.total - discount;
    try {
        const payment = await charge(amount);
        if (payment.success) {
            await saveOrder(order, payment);
        }
        return payment;
    } catch (error) {
        await rollback(order);
        throw error;
    }
}
