export class CheckoutController {
    async handleCheckout(req: any) {
        // Architecture violation: Controller directly accesses database table
        await Database.payments.insert(req.body);
    }
}

export const Database = {
    payments: {
        async insert(data: any) {
            return true;
        }
    }
};
